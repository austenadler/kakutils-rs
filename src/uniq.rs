use kakplugin::{
    get_selections_desc, get_selections_with_desc, set_selections, set_selections_desc, KakError,
    SelectionWithDesc,
};
use regex::Regex;
use std::{
    collections::{hash_map::DefaultHasher, BTreeSet},
    hash::{Hash, Hasher},
};
#[derive(clap::StructOpt, Debug)]
pub struct Options {
    #[clap(index = 1)]
    regex: Option<Regex>,
    #[clap(short, long)]
    ignore_case: bool,
    // TODO: Can we invert a boolean? This name is terrible
    #[clap(short = 'S', long)]
    no_skip_whitespace: bool,
}
pub fn uniq(options: &Options) -> Result<String, KakError> {
    let mut selections = get_selections_with_desc()?;
    // Sort selections so the first element is the unique one, not an arbitrary one based on primary selection
    selections.sort_by_key(|s| s.desc.sort());

    // Set the new selection types
    let new_selections: Vec<Option<SelectionWithDesc>> = selections
        .into_iter()
        // Create a BTreeSet of hashes of selections. This way, string content is not stored, but uniqueness can be determined
        .scan(BTreeSet::new(), |state, s| {
            // Strip whitespace if requested
            let mut key = if options.no_skip_whitespace {
                s.content.as_str()
            } else {
                s.content.trim()
            };

            // If they requested a regex match, set the key to the string slice of that match
            if let Some(regex_match) = (|| {
                let captures = options.regex.as_ref()?.captures(key)?;
                captures
                    .get(1)
                    .or_else(|| captures.get(0))
                    .map(|m| m.as_str())
            })() {
                key = regex_match;
            }

            // Ignore case if requested
            let key = if options.ignore_case {
                key.to_lowercase()
            } else {
                // TODO: Do I really need to clone this?
                key.to_string()
            };

            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);

            // Try inserting to the hash
            if state.insert(hasher.finish()) {
                // True if this is a string we haven't seen before
                Some(Some(s))
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
    let mut new_selections_desc = get_selections_desc()?;
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
