use crate::KakMessage;
use crate::SelectionDesc;
use crate::{kak_response, open_command_fifo};
use alphanumeric_sort::compare_str;
use rand::seq::SliceRandom;
use rand::thread_rng;
use regex::Regex;
use std::cmp::Ordering;
use std::io::Write;
use std::str::FromStr;

#[derive(clap::StructOpt, Debug)]
pub struct SortOptions {
    #[clap(index = 1)]
    regex: Option<Regex>,
    #[clap(short = 's', long)]
    subselections_register: Option<char>,
    // TODO: Can we invert a boolean? This name is terrible
    #[clap(short = 'S', long)]
    no_skip_whitespace: bool,
    #[clap(short, long)]
    lexicographic_sort: bool,
    #[clap(short, long)]
    reverse: bool,
    #[clap(short, long)]
    ignore_case: bool,
}

struct SortableSelection<'a> {
    /// The content of the selection
    content: &'a str,
    /// The string used to compare the content
    content_comparison: &'a str,
    /// Any subselections
    subselections: Vec<&'a str>,
}

/// Gets a Vec of sortable selections with a given list of subselections and descriptions
fn get_sortable_selections_subselections<'a, 'b, 'tmp, S: AsRef<str> + std::fmt::Debug + 'a>(
    sort_options: &'b SortOptions,
    selections: &'a [S],
    selections_desc: &'tmp [S],
    subselections: &'a [S],
    subselections_desc: &'tmp [S],
) -> Result<Vec<SortableSelection<'a>>, KakMessage> {
    eprintln!(
        "All units: {:?}\n{:?}\n{:?}\n{:?}",
        selections, selections_desc, subselections, subselections_desc,
    );
    let mut sortable_selections = selections
        .iter()
        .zip(selections_desc.iter())
        .map(|(s, sd)| {
            Ok((
                to_sortable_selection(s.as_ref(), sort_options),
                SelectionDesc::from_str(sd.as_ref())?,
            ))
        })
        .collect::<Result<Vec<(SortableSelection, SelectionDesc)>, KakMessage>>()?;

    let mut subselections = subselections
        .iter()
        .zip(subselections_desc.iter())
        .map(|(s, sd)| Ok((s.as_ref(), SelectionDesc::from_str(sd.as_ref())?)))
        .collect::<Result<Vec<(&str, SelectionDesc)>, KakMessage>>()?;

    subselections.sort_by(|(_, ssd_a), (_, ssd_b)| ssd_a.cmp(ssd_b));

    // For each selection, check if they contain any subselections
    // If so, add them to the subselections vector
    // TODO: This is O(n^2), but can be made more efficient since subselections is sorted
    for (s, s_desc) in &mut sortable_selections {
        for i in &subselections {
            if s_desc.contains(&i.1) {
                s.subselections.push(i.0.clone());
            }
        }
    }

    sortable_selections.sort_by(|(a, _), (b, _)| {
        // First, check if there are any subselection comparisons to be made
        // If one has more subselections than the other, stop comparing
        for (a_subsel, b_subsel) in a.subselections.iter().zip(b.subselections.iter()) {
            match a_subsel.cmp(b_subsel) {
                // These subselecitons are equal, so we can't do anything
                Ordering::Equal => continue,
                // We found a difference, so return the comparison
                o => return o,
            }
        }

        // No subselections mismatched, so compare the (possibly trimmed) content
        a.content_comparison.cmp(b.content_comparison)
    });

    Ok(sortable_selections.into_iter().map(|(s, _)| s).collect())
}

fn to_sortable_selection<'a, 'b>(
    selection: &'a str,
    sort_options: &'b SortOptions,
) -> SortableSelection<'a> {
    if sort_options.no_skip_whitespace {
        SortableSelection {
            content: selection,
            content_comparison: selection,
            subselections: vec![],
        }
    } else {
        SortableSelection {
            content: selection,
            content_comparison: selection.trim(),
            subselections: vec![],
        }
    }
}

pub fn sort(sort_options: &SortOptions) -> Result<KakMessage, KakMessage> {
    let subselections: Option<(Vec<String>, Vec<String>)> = sort_options
        .subselections_register
        .map::<Result<(Vec<String>, Vec<String>), KakMessage>, _>(|_c| {
            let subselections = kak_response("%val{selections}")?;
            let subselections_desc = kak_response("%val{selections_desc}")?;
            // kak_exec(&format!("exec z",))?;
            Ok((subselections, subselections_desc))
        })
        .transpose()?;
    let selections = kak_response("%val{selections}")?;

    let mut zipped: Vec<SortableSelection<'_>> = match subselections {
        Some((ref subselections, ref subselections_desc)) => {
            let selections_desc = kak_response("%val{selections_desc}")?;

            get_sortable_selections_subselections(
                sort_options,
                &selections,
                &selections_desc,
                subselections,
                subselections_desc,
            )?
        }
        None => selections
            .iter()
            .map(|s| to_sortable_selection(s, sort_options))
            .collect(),
    };

    zipped.sort_by(|a, b| {
        for (a_subselection, b_subselection) in a.subselections.iter().zip(b.subselections.iter()) {
            let comparison = if sort_options.lexicographic_sort {
                a_subselection.cmp(b_subselection)
            } else {
                compare_str(a_subselection, b_subselection)
            };
            match comparison {
                Ordering::Less | Ordering::Greater => return comparison,
                Ordering::Equal => {}
            }
        }

        if sort_options.lexicographic_sort {
            a.content_comparison.cmp(b.content_comparison)
        } else {
            compare_str(a.content_comparison, b.content_comparison)
        }
    });

    let mut f = open_command_fifo()?;

    write!(f, "reg '\"'")?;

    let iter: Box<dyn Iterator<Item = _>> = if sort_options.reverse {
        Box::new(zipped.iter().rev())
    } else {
        Box::new(zipped.iter())
    };

    for i in iter {
        let new_selection = i.content.replace('\'', "''");
        write!(f, " '{}'", new_selection)?;
    }
    write!(f, " ; exec R;")?;

    Ok(KakMessage(
        format!("Sorted {} selections", zipped.len()),
        None,
    ))
}
