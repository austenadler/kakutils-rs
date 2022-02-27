use crate::{kak_response, open_command_fifo, KakMessage};
use std::{
    collections::{hash_map::Entry, HashMap},
    io::Write,
};
#[derive(clap::StructOpt, Debug)]
pub struct UniqOptions {
    #[clap(short, long)]
    ignore_case: bool,
    // TODO: Can we invert a boolean? This name is terrible
    #[clap(short = 'S', long)]
    no_skip_whitespace: bool,
}
pub fn uniq(uniq_options: &UniqOptions) -> Result<KakMessage, KakMessage> {
    let selections = kak_response("%val{selections}")?;

    let mut f = open_command_fifo()?;
    write!(f, "reg '\"'")?;

    for i in selections.iter().scan(HashMap::new(), |state, s| {
        let key = if uniq_options.no_skip_whitespace {
            s
        } else {
            s.trim()
        };

        let key = if uniq_options.ignore_case {
            key.to_lowercase()
        } else {
            // TODO: Do I really need to clone this?
            key.to_string()
        };

        let ret = match state.entry(key) {
            Entry::Vacant(e) => {
                e.insert(());
                s
            }
            Entry::Occupied(_) => {
                // We've seen this selection before, so empty it
                ""
            }
        };

        Some(ret)
    }) {
        let new_selection = i.replace('\'', "''");
        write!(f, " '{}'", new_selection)?;
    }
    write!(f, " ; exec R;")?;

    Ok(KakMessage(
        format!("Uniq {} selections", selections.len()),
        None,
    ))
}
