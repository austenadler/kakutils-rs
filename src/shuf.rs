use crate::{get_selections, set_selections, KakMessage};
use rand::{seq::SliceRandom, thread_rng};
#[derive(clap::StructOpt, Debug)]
pub struct Options;
pub fn shuf(_options: &Options) -> Result<KakMessage, KakMessage> {
    let mut selections = get_selections()?;
    let mut rng = thread_rng();

    selections.shuffle(&mut rng);

    set_selections(selections.iter())?;

    Ok(KakMessage(
        format!("Shuf {} selections", selections.len()),
        None,
    ))
}
