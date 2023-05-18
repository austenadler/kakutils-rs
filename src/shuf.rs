use kakplugin::{get_selections, set_selections, KakError};
use rand::{seq::SliceRandom, thread_rng};
#[derive(clap::Args, Debug)]
pub struct Options;
pub fn shuf(_options: &Options) -> Result<String, KakError> {
    let mut selections = get_selections(None)?;
    let mut rng = thread_rng();

    selections.shuffle(&mut rng);

    set_selections(selections.iter())?;

    Ok(format!("Shuf {} selections", selections.len()))
}
