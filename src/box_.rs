use kakplugin::{
    get_selections_desc, set_selections_desc, AnchorPosition, KakError, SelectionDesc,
};
use std::cmp::{max, min};
#[derive(clap::StructOpt, Debug)]
pub struct Options {
    /// Bounding box mode, which selects the largest box to contain everything
    #[clap(short, long, help = "Select the bonding box of all selections")]
    bounding_box: bool,
    /// Excludes newlines from resulting selection
    #[clap(short, long, help = "Do not include newlines")]
    no_newline: bool,
}

pub fn box_(options: &Options) -> Result<String, KakError> {
    if options.bounding_box {
        // The user requested only the bounding box, so select it first
        set_selections_desc(vec![get_bounding_box(get_selections_desc::<&str>(None)?)
            .ok_or_else(|| {
            KakError::Custom(String::from("Selection is empty"))
        })?])?;
    }

    let ret_selections_desc = boxed_selections(options)?;

    set_selections_desc(ret_selections_desc.iter())?;

    Ok(format!("Boxed {} selection(s)", ret_selections_desc.len()))
}

/// Get the bounding box of some iterator of selections
fn get_bounding_box<SDI>(selections_desc: SDI) -> Option<SelectionDesc>
where
    // SD: AsRef<SelectionDesc>,
    SDI: IntoIterator<Item = SelectionDesc>,
{
    selections_desc
        .into_iter()
        .map(|sd| sd.as_ref().sort())
        .reduce(|acc, sd| SelectionDesc {
            left: AnchorPosition {
                row: min(
                    min(acc.left.row, acc.right.row),
                    min(sd.left.row, sd.right.row),
                ),
                col: min(
                    min(acc.left.col, acc.right.col),
                    min(sd.left.col, sd.right.col),
                ),
            },
            right: AnchorPosition {
                row: max(
                    max(acc.right.row, acc.left.row),
                    max(sd.right.row, sd.left.row),
                ),
                col: max(
                    max(acc.right.col, acc.left.col),
                    max(sd.right.col, sd.left.col),
                ),
            },
        })
}

/// Implementation that converts each selection to a box with the top left corner at min(anchor.col, cursor.col) and bottom right at max(anchor.col, cursor.col)
///
/// Do this by getting each selection, then getting each whole-row (col 0 to col max) and passing the range of whole-rows into helper `to_boxed_selections`
fn boxed_selections(options: &Options) -> Result<Vec<SelectionDesc>, KakError> {
    // The selections we want to box, one per box
    let selections_desc = get_selections_desc::<&str>(None)?;

    let whole_line_selection_command = if options.no_newline {
        // Select everything and only keep non-newlines
        "<a-x>s^[^\\n]+<ret>"
    } else {
        // Select everything and split
        "<a-x><a-s>"
    };

    // Whole-row selections split on newline
    let selections_desc_rows = get_selections_desc(Some(whole_line_selection_command))?;

    Ok(selections_desc
        .iter()
        .map(|sd| {
            // The index in the array that contains the first row in the split lines
            let first_row_idx = selections_desc_rows
                .binary_search_by(|sd_search| sd_search.left.row.cmp(&sd.left.row))
                .map_err(|_| {
                    KakError::Custom(format!(
                        "Selection row {} not found in split rows",
                        sd.left.row
                    ))
                })?;

            // The slice of full row selections
            let sd_rows = selections_desc_rows
                .as_slice()
                // Start at the first (should be only) position in the list with this row
                .take(first_row_idx..)
                .ok_or_else(|| {
                    KakError::Custom(format!(
                        "Rows selections_desc (len={}) has no idx={}",
                        selections_desc_rows.len(),
                        first_row_idx
                    ))
                })?
                // Take row_span rows. For an 8 row selection, get 8 rows, including the one taken before
                .take(..(sd.row_span()))
                .ok_or_else(|| {
                    eprintln!(
                        "rows: {}, row_span: {}, remaining: selections_desc_rows: {}",
                        selections_desc_rows.len(),
                        sd.row_span(),
                        selections_desc_rows.len()
                    );
                    KakError::Custom(String::from(
                        "Selections split on line count mismatch (too few rows)",
                    ))
                })?;

            Ok(to_boxed_selections(sd, sd_rows))
        })
        .collect::<Result<Vec<Vec<SelectionDesc>>, KakError>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<SelectionDesc>>())
}

/// Returns a vec of `selections_desc` of the intersection of the bounding box and the component rows
///
/// This function takes a selection desc, and its whole-row split selections (`<a-x><a-s>`).
/// For each whole-row (col 1 to max col) selection, it finds the intersection between the min col and max col in `selection_desc`
///
/// * `selection_desc` - The base (possibly multiline) `selection_desc`
/// * `selections_desc_rows` - Vec of above `selection_desc` split by line (`<a-x><a-s>`)
fn to_boxed_selections<SD1, SD2>(
    selection_desc: SD1,
    selections_desc_rows: &[SD2],
) -> Vec<SelectionDesc>
where
    SD1: AsRef<SelectionDesc>,
    SD2: AsRef<SelectionDesc>,
{
    let (leftmost_col, rightmost_col) = (
        min(
            selection_desc.as_ref().left.col,
            selection_desc.as_ref().right.col,
        ),
        max(
            selection_desc.as_ref().left.col,
            selection_desc.as_ref().right.col,
        ),
    );

    selections_desc_rows
        .iter()
        .filter_map(|split_sd| {
            // Find the intersection of <row>.<min_col>,<row>.<max_col>
            // If empty, return none. Flatten will not add it to the resulting vec
            split_sd.as_ref().intersect(SelectionDesc {
                left: AnchorPosition {
                    row: split_sd.as_ref().left.row,
                    col: leftmost_col,
                },
                right: AnchorPosition {
                    row: split_sd.as_ref().right.row,
                    col: rightmost_col,
                },
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Selection desc creator
    macro_rules! sd {
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

    // Reversed
    macro_rules! sdr {
        ($b:expr, $d:expr) => {{
            sd!(1, $d, 1, $b)
        }};
        ($a:expr, $b:expr,$c:expr,$d:expr) => {{
            SelectionDesc {
                left: AnchorPosition { row: $c, col: $d },
                right: AnchorPosition { row: $a, col: $b },
            }
        }};
    }

    #[test]
    fn test_get_bounding_box() {
        assert!(get_bounding_box(Vec::new()).is_none());
        assert_eq!(get_bounding_box(vec![sd!(0, 1)]).unwrap(), sd!(0, 1));
        assert_eq!(
            get_bounding_box(vec![sd!(0, 0, 8, 2), sd!(1, 15, 9, 3)]).unwrap(),
            sd!(0, 0, 9, 15)
        );
        assert_eq!(get_bounding_box(vec![sdr!(0, 1)]).unwrap(), sd!(0, 1));
        assert_eq!(
            get_bounding_box(vec![sdr!(0, 0, 8, 2), sdr!(1, 15, 9, 3)]).unwrap(),
            sd!(0, 0, 9, 15)
        );
    }
}
