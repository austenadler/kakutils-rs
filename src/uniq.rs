use crate::utils;
use kakplugin::{
    get_selections_desc, get_selections_with_desc, set_selections, set_selections_desc, KakError,
    SelectionWithDesc,
};
use regex::Regex;
use std::collections::BTreeSet;

#[derive(clap::StructOpt, Debug)]
pub struct Options {
    #[clap(index = 1, help = "Optional regex to compare unique elements")]
    regex: Option<Regex>,
    #[clap(short, long, help = "Ignore the case when comparing")]
    ignore_case: bool,
    // TODO: Can we invert a boolean? This name is terrible
    #[clap(short = 'S', long, help = "Do not skip whitespace when comparing")]
    no_skip_whitespace: bool,
}
pub fn uniq(options: &Options) -> Result<String, KakError> {
    let mut selections = get_selections_with_desc(None)?;
    // Sort selections so the first element is the unique one, not an arbitrary one based on primary selection
    selections.sort_by_key(|s| s.desc.sort());

    // Set the new selection types
    let new_selections: Vec<Option<SelectionWithDesc>> = selections
        .into_iter()
        // Create a BTreeSet of hashes of selections. This way, string content is not stored, but uniqueness can be determined
        .scan(BTreeSet::new(), |state, sd| {
            let hash = utils::get_hash(
                &sd.content,
                !options.no_skip_whitespace,
                options.regex.as_ref(),
                options.ignore_case,
            );

            // Try inserting to the hash
            if state.insert(hash) {
                // True if this is a string we haven't seen before
                Some(Some(sd))
            } else {
                // Nothing was inserted because we already saw this string
                // Return Some(None) so the iterator can continue
                Some(None)
            }
        })
        .collect();

    set_selections(new_selections.iter().map(|i| match i {
        Some(s) => &s.content,
        None => "",
    }))?;

    // Deselect all `None` strings (all rows that have been seen before)
    let mut new_selections_desc = get_selections_desc::<&str>(None)?;
    new_selections_desc.sort();
    set_selections_desc(
        // Refresh seelections_desc because positions have changed
        new_selections_desc
            .iter()
            .zip(new_selections.iter())
            // If the string was emptied (None), then do not set `sd`
            .filter_map(|(sd, s)| if s.is_some() { Some(sd) } else { None }),
    )?;

    let old_count = new_selections.len();
    let new_count = new_selections.iter().flatten().count();

    Ok(format!(
        "{} unique selections out of {}",
        new_count, old_count
    ))
}
