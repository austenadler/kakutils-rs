use itertools::Itertools;
use kakplugin::{get_selections_desc_unordered, set_selections_desc, KakError, SelectionDesc};

#[derive(Debug, clap::Args)]
pub struct Options {
    #[clap(index = 1, value_parser = clap::value_parser!(u16).range(2..))]
    keep_every: u16,
}

pub fn keep_every(options: &Options) -> Result<String, KakError> {
    let selections_desc = get_selections_desc_unordered(None)?;
    let old_count = selections_desc.len();

    let new_selections_desc = apply(options, &selections_desc);

    set_selections_desc(new_selections_desc.iter())?;

    let new_count = new_selections_desc.len();

    Ok(format!("{} kept from {}", new_count, old_count))
}

fn apply(options: &Options, selections_desc: &[SelectionDesc]) -> Vec<SelectionDesc> {
    selections_desc
        .iter()
        .chunks(options.keep_every.into())
        .into_iter()
        .flat_map(|mut it| it.next())
        .copied()
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    // Selection desc creator
    macro_rules! sd {
        ($a:expr) => {{
            sd!($a, $a)
        }};
        ($b:expr, $d:expr) => {{
            sd!(1, $b, 1, $d)
        }};
        ($a:expr, $b:expr,$c:expr,$d:expr) => {{
            SelectionDesc {
                left: AnchorPosition { row: $a, col: $b },
                right: AnchorPosition { row: $c, col: $d },
            }
        }};
    }

    use kakplugin::{AnchorPosition, SelectionDesc};

    use super::*;
    #[test]
    fn test() {
        assert_eq!(apply(&Options { keep_every: 2 }, &[sd!(1),]), &[sd!(1),]);
        assert_eq!(
            apply(
                &Options { keep_every: 2 },
                &[sd!(1), sd!(2), sd!(3), sd!(4), sd!(5), sd!(6), sd!(7),]
            ),
            &[sd!(1), sd!(3), sd!(5), sd!(7),]
        );
    }
}
