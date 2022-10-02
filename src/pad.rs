use crate::utils::split_newlines;
use evalexpr::{eval, Value};
use kakplugin::{get_selections, open_command_fifo, set_selections, KakError, Selection};
use std::{borrow::Cow, io::Write};

#[derive(clap::StructOpt, Debug)]
pub struct Options {
    #[clap(index = 1, help = "Pad with this char", default_value = "0")]
    fill: char,
    #[clap(short, long, help = "Pad on the right instead of the left")]
    right: bool,
}

pub fn pad(options: &Options) -> Result<String, KakError> {
    let selections = get_selections(None)?;
    let selections_trailing_split: Vec<(&str, &str, &str)> = selections
        .iter()
        // We don't want leading or trailing newlines to count
        .map(|s| split_newlines(s))
        .collect();

    // The max length of selections with newlines split off
    let max_len = selections_trailing_split
        .iter()
        .map(|(_, s, _)| s.len())
        .max()
        .ok_or(KakError::CustomStatic("No selections"))?;

    let mut num_padded: usize = 0;
    let num_total = selections.len();

    set_selections(selections_trailing_split.iter().zip(selections.iter()).map(
        |((leading_newlines, s, trailing_newlines), orig_s)| match max_len.checked_sub(s.len()) {
            Some(0) | None => Cow::Borrowed(orig_s.as_str()),
            Some(len) => {
                num_padded += 1;
                let fill = options.fill.to_string().repeat(len);
                let mut ret = leading_newlines.to_string();
                if options.right {
                    ret.push_str(s);
                    ret.push_str(&fill);
                } else {
                    ret.push_str(&fill);
                    ret.push_str(s);
                }
                ret.push_str(trailing_newlines);
                Cow::Owned(ret)
            }
        },
    ))?;

    Ok(format!(
        "Padded {num_padded} selections ({num_total} total)",
    ))
}
