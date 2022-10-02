use alphanumeric_sort::compare_str;
use kakplugin::{self, get_selections_with_desc, open_command_fifo, KakError, SelectionWithDesc};
use regex::Regex;
use std::{borrow::Cow, cmp::Ordering, io::Write};

#[derive(clap::StructOpt, Debug)]
pub struct Options {
    #[clap(index = 1, help = "Optional regex comparison key")]
    regex: Option<Regex>,
    #[clap(
        short = 's',
        long,
        help = "Optional register for using subselection matching"
    )]
    subselections_register: Option<char>,
    // TODO: Can we invert a boolean? This name is terrible
    #[clap(short = 'S', long, parse(try_from_str = invert_bool), default_value_t, help = "Do not treat trimmed value of selections when sorting")]
    no_skip_whitespace: bool,
    #[clap(short = 'L', long, help = "Do not sort numbers lexicographically")]
    no_lexicographic_sort: bool,
    #[clap(short, long, help = "Reverse sorting")]
    reverse: bool,
    #[clap(short, long, help = "Ignore case when sorting")]
    ignore_case: bool,
}

fn invert_bool(s: &str) -> Result<bool, &'static str> {
    // Invert the boolean
    match s {
        "false" => Ok(true),
        "true" => Ok(false),
        _ => Err("Unparsable boolean value"),
    }
}

struct SortableSelection<'a> {
    /// The content of the selection
    selection: &'a SelectionWithDesc,
    /// The string used to compare the content
    content_comparison: Cow<'a, str>,
    /// Any subselections
    subselections: Vec<&'a str>,
}

/// Gets a Vec of sortable selections with a given list of subselections and descriptions
/// TODO: Implement sort by subselection
// fn get_sortable_selections_subselections<'a, 'b, 'tmp, S: AsRef<str> + std::fmt::Debug + 'a>(
//     options: &'b Options,
//     selections: Vec<SelectionWithDesc>,
//     subselections: Vec<SelectionWithDesc>,
// ) -> Result<Vec<SortableSelection<'a>>, KakMessage> {
//     let mut sortable_selections = selections
//         .iter()
//         .zip(selections_desc.iter())
//         .map(|(s, sd)| {
//             Ok((
//                 to_sortable_selection(s.as_ref(), options),
//                 SelectionDesc::from_str(sd.as_ref())?,
//             ))
//         })
//         .collect::<Result<Vec<(SortableSelection, SelectionDesc)>, KakMessage>>()?;

//     let mut subselections = subselections
//         .iter()
//         .zip(subselections_desc.iter())
//         // Bind selection with associated description
//         // Sort descriptions so left is always <= right
//         .map(|(s, sd)| Ok((s.as_ref(), SelectionDesc::from_str(sd.as_ref())?.sort())))
//         .collect::<Result<Vec<(&str, SelectionDesc)>, KakMessage>>()?;

//     // Sort subselections by description
//     subselections.sort_by(|(_, ssd_a), (_, ssd_b)| ssd_a.cmp(ssd_b));

//     // For each selection, check if they contain any subselections
//     // If so, add them to the subselections vector
//     // TODO: This is O(n^2), but can be made more efficient since subselections is sorted
//     for (s, s_desc) in &mut sortable_selections {
//         for i in &subselections {
//             if s_desc.contains(&i.1) {
//                 s.subselections.push(i.0);
//             }
//         }
//     }

//     sortable_selections.sort_by(|(a, _), (b, _)| {
//         // First, check if there are any subselection comparisons to be made
//         // If one has more subselections than the other, stop comparing
//         for (a_subsel, b_subsel) in a.subselections.iter().zip(b.subselections.iter()) {
//             match a_subsel.cmp(b_subsel) {
//                 // These subselections are equal, so we can't do anything
//                 Ordering::Equal => continue,
//                 // We found a difference, so return the comparison
//                 o => return o,
//             }
//         }

//         // No subselections mismatched, so compare the (possibly trimmed) content
//         a.content_comparison.cmp(b.content_comparison)
//     });

//     Ok(sortable_selections.into_iter().map(|(s, _)| s).collect())
// }

fn to_sortable_selection<'a, 'b>(
    selection: &'a SelectionWithDesc,
    options: &'b Options,
) -> SortableSelection<'a> {
    SortableSelection {
        selection,
        // TODO: Properly use Cow
        content_comparison: crate::utils::get_key(
            &selection.content,
            !options.no_skip_whitespace,
            options.regex.as_ref(),
            options.ignore_case,
        )
        .into(),
        subselections: vec![],
    }
}

pub fn sort(options: &Options) -> Result<String, KakError> {
    // subselections is Some if the user requests it in subselections_register
    // It will "exec z" to restore the selections before setting selections
    // If subselections is None, "exec z" is not called
    let subselections: Option<Vec<SelectionWithDesc>> = options
        .subselections_register
        .map::<Result<_, KakError>, _>(|c| {
            let subselections = get_selections_with_desc(None)?;
            kakplugin::cmd(&format!("exec {}", c))?;
            Ok(subselections)
        })
        .transpose()?;
    let selections = get_selections_with_desc(None)?;

    let mut zipped: Vec<SortableSelection<'_>> = match (&options.regex, &subselections) {
        (Some(_), Some(_)) => {
            return Err(KakError::Custom(
                "Cannot pass regex and subselections register".to_string(),
            ))
        }
        (None, None) => {
            // Do a regular sort on the content
            selections
                .iter()
                .map(|s| to_sortable_selection(s, options))
                .collect()
        }
        (Some(_regex), None) => {
            // Sort based on the regular expression
            selections
                .iter()
                .map(|s| to_sortable_selection(s, options))
                .collect()

            // TODO: Figure out if this is fine
            // selections
            //     .iter()
            //     .map(|s| {
            //         let mut sortable_selection = to_sortable_selection(s, options);
            //         if let Some(regex_match) = (|| {
            //             let captures = regex.captures(sortable_selection.content_comparison)?;
            //             captures
            //                 .get(1)
            //                 .or_else(|| captures.get(0))
            //                 .map(|m| m.as_str())
            //         })() {
            //             sortable_selection.content_comparison = regex_match;
            //         }

            //         sortable_selection
            //     })
            //     .collect()
        }
        (None, _) => {
            // Sort based on subselections
            return Err(KakError::NotImplemented(
                "Sort by subselection is not yet implemented",
            ));
        }
    };

    zipped.sort_by(|a, b| {
        // First, try sorting by subselection. This won't iterate anything if either is None (regex and default mode)
        for (a_subselection, b_subselection) in a.subselections.iter().zip(b.subselections.iter()) {
            let comparison = if options.no_lexicographic_sort {
                a_subselection.cmp(b_subselection)
            } else {
                compare_str(&a_subselection, &b_subselection)
            };

            // If the comparison is not equal, stop here
            if comparison != Ordering::Equal {
                return comparison;
            }
        }

        // Otherwise, default to comparing the content
        if options.no_lexicographic_sort {
            a.content_comparison.cmp(&b.content_comparison)
        } else {
            compare_str(&a.content_comparison, &b.content_comparison)
        }
    });

    let mut f = open_command_fifo()?;

    write!(f, "reg '\"'")?;

    let iter: Box<dyn Iterator<Item = _>> = if options.reverse {
        Box::new(zipped.iter().rev())
    } else {
        Box::new(zipped.iter())
    };

    for i in iter {
        let new_selection = i.selection.content.replace('\'', "''");
        write!(f, " '{}'", new_selection)?;
    }
    write!(f, " ; exec R;")?;
    f.flush()?;

    Ok(format!("Sorted {} selections", zipped.len()))
}
