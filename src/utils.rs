// use kakplugin::Selection;
use regex::Regex;
use std::{
    borrow::Cow,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

/// Gets a key out of a selection
///
/// # Examples
///
/// ```
/// assert_eq!(get_key("  asdf\n", false, None, false), "asdf\n");
/// assert_eq!(get_key("  asdf\n", true, None, false), "  asdf\n");
/// assert_eq!(get_key("  as1f\n", false, Some("\w+"), false), "as");
/// assert_eq!(get_key("  aS1F\n", false, Some("\w+"), true), "as1f");
/// ```
pub fn get_key<'sel>(
    selection: &'sel str,
    preserve_whitespace: bool,
    regex: Option<&Regex>,
    ignore_case: bool,
) -> Cow<'sel, str> {
    // Strip whitespace if requested
    let mut key = if preserve_whitespace {
        // TODO: Does this need to be swapped?
        selection
    } else {
        selection.trim()
    };

    // If they requested a regex match, set the key to the string slice of that match
    if let Some(regex_match) = (|| {
        // let captures = regex.as_ref()?.captures(&key)?;
        let captures = regex.as_ref()?.captures(key)?;
        captures
            .get(1)
            .or_else(|| captures.get(0))
            .map(|m| m.as_str())
    })() {
        key = regex_match;
        // Cow::Borrowed(regex_match)
    }

    // Ignore case if requested
    if ignore_case {
        // Lowercase at the end to not mangle regex
        // TODO: Do not allocate if it is already lowercased
        // Need to_lowercase(&self) -> Cow<str>
        if !key.as_bytes().iter().any(u8::is_ascii_uppercase) {
            Cow::Borrowed(key)
        } else {
            Cow::Owned(key.to_ascii_lowercase())
        }
    } else {
        Cow::Borrowed(key)
    }
}

/// Get a key out of a selection based on options
pub fn get_hash(
    // TODO: Accept any Into<AsRef<Selection>>
    selection: &str,
    preserve_whitespace: bool,
    regex: Option<&Regex>,
    ignore_case: bool,
) -> u64 {
    let mut hasher = DefaultHasher::new();

    get_key(&selection, preserve_whitespace, regex, ignore_case).hash(&mut hasher);

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
