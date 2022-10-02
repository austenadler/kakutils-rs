use kakplugin::Selection;
use regex::Regex;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub fn get_key(
    // TODO: Use Cow
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
pub fn get_hash(
    // TODO: Accept any Into<AsRef<Selection>>
    selection: &Selection,
    skip_whitespace: bool,
    regex: Option<&Regex>,
    ignore_case: bool,
) -> u64 {
    let mut hasher = DefaultHasher::new();

    get_key(selection, skip_whitespace, regex, ignore_case).hash(&mut hasher);

    hasher.finish()
}

/// Splits an `&str` into (`leading_newlines`, `string_value`, `trailing_newlines`)
///
/// # Examples
///
/// ```
/// assert_eq!(split_newlines("asdf\n"), ("", "asdf", "\n"));
/// assert_eq!(split_newlines("asdf\n\nhjk\n"), ("", "asdf\n\nhjk", "\n"));
/// assert_eq!(split_newlines("\nasdf\n\nhjk\n"), ("\n", "asdf\n\nhjk", "\n"));
/// assert_eq!(split_newlines("asdf"), ("", "asdf", ""));
/// assert_eq!(split_newlines("\n\n\nasdf"), ("\n\n\n", "asdf", ""));
/// assert_eq!(split_newlines(""), ("", "", ""));
/// ```
pub fn split_newlines(s: &'_ str) -> (&'_ str, &'_ str, &'_ str) {
    let (leading_newlines, s) = s.find(|c| c != '\n').map_or(("", s), |idx| s.split_at(idx));

    let (s, trailing_newlines) = s
        .rfind(|c| c != '\n')
        .map_or((s, ""), |idx| s.split_at(idx + 1));

    (leading_newlines, s, trailing_newlines)
}
