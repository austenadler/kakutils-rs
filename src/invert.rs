use kakplugin::{
    get_selections_desc, set_selections_desc, types::MaybeSplit, AnchorPosition, KakError,
    SelectionDesc,
};
use std::{fs, str::FromStr};
#[derive(clap::StructOpt, Debug)]
pub struct Options {
    #[clap(short, long, help = "Do not include newlines")]
    no_newline: bool,
}

pub fn invert(options: &Options) -> Result<String, KakError> {
    // The selections to invert
    let mut split_selections_desc: Vec<(usize, Vec<SelectionDesc>)> = {
        // Split by multiline so subtraction is defined (see below)
        // Group by row, so for a given document row, subtraction can iterate over the Vec
        get_selections_desc(Some("<a-s>"))?
            .group_by(|a, b| a.left.row == b.left.row)
            .map(|sds| (sds[0].left.row, sds.to_vec()))
            .collect()
    };

    let count_selections = split_selections_desc.len();

    let whole_document_selection_command = if options.no_newline {
        // Select everything and only keep non-newlines
        "%s^[^\\n]+<ret>"
    } else {
        // Select everything and split
        "%<a-s>"
    };

    let document_descs: Vec<SelectionDesc> = {
        // Every line in the document as a selectiondesc
        // Split by line because subtracting cross-multiline is not always defined for multiline selection descs (ex: 1.1,3.3 - 2.1,3.3 = 1.1,1.<?>)
        get_selections_desc(Some(whole_document_selection_command))?
            .into_iter()
            // dd - The full row selectiondesc, spanning from col 1 to the rightmost col, for every row in the file
            .map(|dd: SelectionDesc| {
                // For every line, if there are selections to subtract, subtract them all
                match split_selections_desc
                    .binary_search_by(|sd_search| sd_search.0.cmp(&dd.left.row))
                {
                    Ok(idx) => {
                        // There is at least one SelectionDesc that needs to be subtracted from dd
                        subtract_all_selections_desc(dd, split_selections_desc.remove(idx).1)
                    }
                    Err(_) => {
                        // There are no SelectionDesc entries that need to be subtracted from this row. return it
                        vec![dd]
                    }
                }
            })
            .flatten()
            .collect()
    };

    set_selections_desc(document_descs.iter())?;

    kakplugin::cmd("exec '<a-_>'")?;

    Ok(format!("Inverted {} selections", count_selections))
}

/// Subtract an iterator of `SelectionDesc`s from a given SelectionDesc
///
/// This returns a `Vec` because splitting in the middle can create two `SelectionDesc`s
///
/// * `selection_desc` - The primary SelectionDesc to be subtracted from
/// * `selections_desc_to_subtract` - `Vec` of `SelectionDesc`s from `sd`. Must be an owned `Vec` because it needs to be sorted
fn subtract_all_selections_desc<SD1, SD2>(
    selection_desc: SD1,
    mut selections_desc_to_subtract: Vec<SD2>,
) -> Vec<SelectionDesc>
where
    SD1: AsRef<SelectionDesc>,
    SD2: AsRef<SelectionDesc> + Ord,
{
    // If it is sorted, the selections to subtract will be in left to right order
    // This way, we can store just the rightmost `selection_desc`
    selections_desc_to_subtract.sort();

    let mut rightmost_selection_desc: SelectionDesc = selection_desc.as_ref().clone();
    let mut ret = vec![];

    for sd in selections_desc_to_subtract {
        match rightmost_selection_desc.as_ref().subtract(sd.as_ref()) {
            MaybeSplit::Nothing => {
                // Subtraction yeilded no selections. This selection desc needs to be excluded
                return ret;
            }
            MaybeSplit::Just(sd) => {
                // There was a successful split, but it was a prefix/suffix truncation
                // We don't know if more selections will cut this selection, so continue
                // TODO: Replace Just with JustLeft and JustRight?
                rightmost_selection_desc = sd.as_ref().clone();
            }
            MaybeSplit::JustTwo(sda, sdb) => {
                // There was a split in the middle of the selection
                // Put the left half into the return vector and keep checking if the right half needs more work
                ret.push(sda);
                rightmost_selection_desc = sdb;
            }
        }
    }

    // If we got here, the iterator ran out of things to subtract from us
    // Push whatever is in the rightmost selection desc and continue
    ret.push(rightmost_selection_desc);

    ret
}
