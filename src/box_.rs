use kakplugin::{
    get_selections_desc, set_selections_desc, AnchorPosition, KakError, SelectionDesc,
};
use std::cmp::{max, min};
#[derive(clap::StructOpt, Debug)]
pub struct Options {
    // /// Bounding box mode, which selects the largest box to contain everything
// #[clap(short, long, help = "Select the bonding box of all selections")]
// bounding_box: bool,
// /// Allow selecting trailing newlines
// #[clap(short, long, help = "Allow selecting trailing newlines")]
// preserve_newlines: bool,
}

pub fn box_(options: &Options) -> Result<String, KakError> {
    // TODO: Research if having multiple bounding boxes makes sense
    // let ret_selection_descs = if options.bounding_box {
    //     // Get the bounding box and select it
    //     bounding_box(options)?
    // } else {
    //     // Get a box per selection
    //     todo!("Implement per_selection(options: &Options);");
    // };

    let ret_selection_descs = bounding_box(options)?;

    set_selections_desc(ret_selection_descs.iter())?;

    Ok(format!("Box {} selections", ret_selection_descs.len()))
}

fn bounding_box(_options: &Options) -> Result<Vec<SelectionDesc>, KakError> {
    let selection_descs: Vec<SelectionDesc> = get_selections_desc()?
        .iter()
        // TODO: Do they need to be sorted?
        .map(|sd| sd.sort())
        .collect();

    let (leftmost_col, rightmost_col) = selection_descs
        .iter()
        // Extract the columns so they can be reduced
        // Make the left one be the smaller one in case the first one is max or min (reduce function would not be called)
        .map(|sd| {
            (
                min(sd.left.col, sd.right.col),
                max(sd.left.col, sd.right.col),
            )
        })
        // Get the smallest column or row
        .reduce(|(leftmost_col, rightmost_col), (left, right)| {
            (
                min(leftmost_col, min(left, right)),
                max(rightmost_col, min(left, right)),
            )
        })
        .ok_or_else(|| KakError::Custom(String::from("Selection is empty")))?;

    // Now, split on newline
    // TODO: Should I use <a-s>?
    // kakplugin::cmd(&format!("exec 'S\\n<ret>'"))?;
    kakplugin::cmd(&format!("exec '<a-s>'"))?;
    // TODO: Here is where I might want selections to check if they end in newline

    let mut ret_selection_descs: Vec<SelectionDesc> = vec![];

    let split_selection_descs: Vec<SelectionDesc> =
        get_selections_desc()?.iter().map(|sd| sd.sort()).collect();

    for sd in &split_selection_descs {
        if sd.left.col > rightmost_col || sd.right.col < leftmost_col {
            // If this selection is out of bounds, exclude this line
            continue;
        }

        ret_selection_descs.push(SelectionDesc {
            left: AnchorPosition {
                row: sd.left.row,
                col: leftmost_col,
            },
            right: AnchorPosition {
                row: sd.right.row,
                col: rightmost_col,
            },
        });
        // The left- and right-most col

        // let subselection_descs
    }

    set_selections_desc(&ret_selection_descs[..])?;

    Ok(ret_selection_descs)
}
