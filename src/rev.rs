use kakplugin::{get_selections, set_selections, KakError};
#[derive(clap::StructOpt, Debug)]
pub struct Options;

pub fn rev(_options: &Options) -> Result<String, KakError> {
    let selections = get_selections(None)?;

    set_selections(selections.iter().rev())?;

    Ok(format!("Reversed {} selections", selections.len()))
}
