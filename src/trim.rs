use crate::{get_selections, open_command_fifo, KakMessage};
use std::io::Write;

#[derive(clap::StructOpt, Debug)]
pub struct Options {
    #[clap(short, long)]
    left: bool,
    #[clap(short, long)]
    right: bool,
    #[clap(short, long)]
    no_preserve_newline: bool,
}

pub fn trim(options: &Options) -> Result<KakMessage, KakMessage> {
    let selections = get_selections()?;

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
            s + "\n"
        } else {
            new_string.to_string()
        }
    }) {
        write!(f, " '{}'", i.replace('\'', "''"))?;
    }
    write!(f, " ; exec R;")?;
    f.flush()?;

    Ok(KakMessage(
        format!(
            "Trimmed {} selections ({} changed)",
            num_selections, num_trimmed
        ),
        None,
    ))
}
