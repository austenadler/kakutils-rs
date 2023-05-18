use kakplugin::{get_selections, open_command_fifo, KakError};
use std::io::Write;

#[derive(clap::Args, Debug)]
pub struct Options {
    #[clap(short, long, help = "Trim from left")]
    left: bool,
    #[clap(short, long, help = "Trim right side")]
    right: bool,
    #[clap(
        short,
        long,
        help = "If selection ends in a newline, do not add the newline back after trimming"
    )]
    no_preserve_newline: bool,
}

pub fn trim(options: &Options) -> Result<String, KakError> {
    let selections = get_selections(None)?;

    let mut f = open_command_fifo()?;
    write!(f, "reg '\"'")?;

    let mut num_trimmed: usize = 0;
    let num_selections = selections.len();

    for i in selections.into_iter().map(|s| {
        let new_string = match (options.left, options.right) {
            (true, true) | (false, false) => {
                // Either they specified both, or neither
                s.trim()
            }
            (true, false) => s.trim_start(),
            (false, true) => s.trim_end(),
        };

        if s.len() != new_string.len() {
            num_trimmed = num_trimmed.saturating_add(1);
        }

        if !options.no_preserve_newline && s.ends_with('\n') {
            new_string.to_owned() + "\n"
        } else {
            new_string.to_owned()
        }
    }) {
        write!(f, " '{}'", i.replace('\'', "''"))?;
    }
    write!(f, " ; exec R;")?;
    f.flush()?;

    Ok(format!(
        "Trimmed {} selections ({} changed)",
        num_selections, num_trimmed
    ))
}
