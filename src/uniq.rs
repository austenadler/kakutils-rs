use crate::{kak_response, open_command_fifo, KakMessage};
use std::{
    collections::{hash_map::DefaultHasher, BTreeSet},
    hash::{Hash, Hasher},
    io::Write,
};
#[derive(clap::StructOpt, Debug)]
pub struct Options {
    #[clap(short, long)]
    ignore_case: bool,
    // TODO: Can we invert a boolean? This name is terrible
    #[clap(short = 'S', long)]
    no_skip_whitespace: bool,
}
pub fn uniq(options: &Options) -> Result<KakMessage, KakMessage> {
    let selections = kak_response("%val{selections}")?;

    let mut f = open_command_fifo()?;
    write!(f, "reg '\"'")?;

    for i in selections.iter().scan(BTreeSet::new(), |state, s| {
        let key = if options.no_skip_whitespace {
            s
        } else {
            s.trim()
        };

        let key = if options.ignore_case {
            key.to_lowercase()
        } else {
            // TODO: Do I really need to clone this?
            key.to_string()
        };

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);

        Some(if state.insert(hasher.finish()) {
            // True if this is a new line
            s
        } else {
            // Nothing was inserted because we already saw this line
            ""
        })
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
