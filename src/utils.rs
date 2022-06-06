use kakplugin::Selection;
use regex::Regex;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub(crate) fn get_key(
    selection: &Selection,
    skip_whitespace: bool,
    regex: Option<&Regex>,
    ignore_case: bool,
) -> String {
    // Strip whitespace if requested
    let mut key = if skip_whitespace {
        selection.as_str()
    } else {
        selection.trim()
    };

    // If they requested a regex match, set the key to the string slice of that match
    if let Some(regex_match) = (|| {
        let captures = regex.as_ref()?.captures(key)?;
        captures
            .get(1)
            .or_else(|| captures.get(0))
            .map(|m| m.as_str())
    })() {
        key = regex_match;
    }

    // Ignore case if requested
    // Lowercase at the end to not mangle regex
    if ignore_case {
        key.to_lowercase()
    } else {
        // TODO: Do not perform an allocation here
        key.to_string()
    }
}

/// Get a key out of a selection based on options
pub(crate) fn get_hash(
    selection: &Selection,
    skip_whitespace: bool,
    regex: Option<&Regex>,
    ignore_case: bool,
) -> u64 {
    let mut hasher = DefaultHasher::new();

    get_key(selection, skip_whitespace, regex, ignore_case).hash(&mut hasher);

    hasher.finish()
}
