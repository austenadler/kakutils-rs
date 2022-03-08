use crate::{kak_response, open_command_fifo, KakMessage};
use rand::{seq::SliceRandom, thread_rng};
use std::io::Write;
#[derive(clap::StructOpt, Debug)]
pub struct Options;
pub fn shuf(_options: &Options) -> Result<KakMessage, KakMessage> {
    let mut selections = kak_response("%val{selections}")?;
    let mut rng = thread_rng();

    selections.shuffle(&mut rng);

    let mut f = open_command_fifo()?;
    write!(f, "reg '\"'")?;

    for i in &selections {
        let new_selection = i.replace('\'', "''");
        write!(f, " '{}'", new_selection)?;
    }
    write!(f, " ; exec R;")?;
    f.flush()?;

    Ok(KakMessage(
        format!("Shuf {} selections", selections.len()),
        None,
    ))
}
