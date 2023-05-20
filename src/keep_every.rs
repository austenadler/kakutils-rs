use itertools::Itertools;
use kakplugin::{get_selections_desc_unordered, set_selections_desc, KakError};

#[derive(Debug, clap::Args)]
pub struct Options {
    #[clap(index = 1, value_parser = clap::value_parser!(u16).range(2..))]
    keep_every: u16,
}

pub fn keep_every(options: &Options) -> Result<String, KakError> {
    let old_selections_desc = get_selections_desc_unordered(None)?;

    let mut new_count = 0;
    set_selections_desc(
        old_selections_desc
            .iter()
            .chunks(options.keep_every.into())
            .into_iter()
            .flat_map(|mut it| {
                // Only keep the first selection from each chunk
                new_count += 1;
                it.next()
            }),
    )?;

    Ok(format!(
        "{} kept from {}",
        new_count,
        old_selections_desc.len()
    ))
}
