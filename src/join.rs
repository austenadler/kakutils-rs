use kakplugin::{get_selections_desc_unordered, set_selections_desc, KakError};

#[derive(clap::StructOpt, Debug)]
pub struct Options;

pub fn join(_options: &Options) -> Result<String, KakError> {
    set_selections_desc(
        get_selections_desc_unordered(None)?
            .into_iter()
            .reduce(|acc, sd| acc.bounding_selection(sd)),
    )?;

    Ok(format!("Joined all selections"))
}
