use crate::{get_var, KakMessage};
// use shellwords::ShellWordsIterator;
use std::{
    fmt,
    fs::{self, File, OpenOptions},
    io::{BufWriter, Write},
    str::FromStr,
};

pub type Selection = String;

#[derive(PartialEq, Eq, Debug)]
pub struct SelectionWithDesc {
    pub content: Selection,
    pub desc: SelectionDesc,
}

#[derive(PartialEq, Eq, Debug)]
pub struct SelectionWithSubselections {
    pub selection: SelectionWithDesc,
    pub subselections: Vec<SelectionWithDesc>,
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug)]
pub struct SelectionDesc {
    pub left: AnchorPosition,
    pub right: AnchorPosition,
}

impl SelectionDesc {
    #[must_use]
    pub fn sort(&self) -> Self {
        if self.left < self.right {
            // left anchor is first
            Self {
                left: self.left.clone(),
                right: self.right.clone(),
            }
        } else {
            // right anchor is first
            Self {
                left: self.right.clone(),
                right: self.left.clone(),
            }
        }
    }

    #[must_use]
    pub fn contains(&self, b: &Self) -> bool {
        // Cursor and anchor can be flipped. Set a.0 to be leftmost cursor
        let sorted_a = self.sort();
        let sorted_b = b.sort();

        sorted_b.left >= sorted_a.left && sorted_b.right <= sorted_a.right
    }
}

impl fmt::Display for SelectionDesc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.left, self.right)
    }
}

impl FromStr for SelectionDesc {
    type Err = KakMessage;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (left, right) = s.split_once(',').ok_or_else(|| {
            KakMessage(
                "Could not parse position".to_string(),
                Some(format!("Could not parse as position: {}", s)),
            )
        })?;

        Ok(Self {
            left: AnchorPosition::from_str(left)?,
            right: AnchorPosition::from_str(right)?,
        })
    }
}

#[derive(PartialOrd, PartialEq, Clone, Eq, Ord, Debug)]
pub struct AnchorPosition {
    pub row: usize,
    pub col: usize,
}
impl fmt::Display for AnchorPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.row, self.col)
    }
}

impl FromStr for AnchorPosition {
    type Err = KakMessage;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (left, right) = s.split_once('.').ok_or_else(|| {
            KakMessage(
                "Could not parse position".to_string(),
                Some(format!("Could not parse as position: {}", s)),
            )
        })?;
        Ok(Self {
            row: usize::from_str(left)?,
            col: usize::from_str(right)?,
        })
    }
}

/// # Errors
///
/// Will return `Err` if command fifo could not be opened, read from, or written to
pub fn get_selections() -> Result<Vec<Selection>, KakMessage> {
    response("%val{selections}")
}

/// # Errors
///
/// Will return `Err` if command fifo could not be opened, read from, or written to
pub fn get_selections_desc() -> Result<Vec<SelectionDesc>, KakMessage> {
    response("%val{selections_desc}")?
        .iter()
        .map(|sd| SelectionDesc::from_str(sd))
        .collect::<Result<Vec<_>, KakMessage>>()
}

// pub fn get_selections_with_subselections(
//     register: &str,
// ) -> Result<Vec<SelectionWithSubselections>, KakMessage> {
//     // TODO: Escape register
//     let subselections = get_selections_with_desc()?;
//     exec(format!("\"{}z", register.replace('\'', "''")))?;
//     let selections = get_selections_with_desc()?;

//     for sel in selections {
//         for i in subselections {}
//     }
// }
/// # Errors
///
/// Will return `Err` if command fifo could not be opened, read from, or written to,
/// or if `selections.len() != selections_desc.len`
pub fn get_selections_with_desc() -> Result<Vec<SelectionWithDesc>, KakMessage> {
    let mut selections = get_selections()?;
    let selections_desc = get_selections_desc()?;

    if selections.len() != selections_desc.len() {
        return Err(KakMessage(
            "Internal error".to_string(),
            Some(format!(
                "Counts for selections={}, selections_desc={}",
                selections.len(),
                selections_desc.len()
            )),
        ));
    }

    // Kakoune prints selections in file order, but prints selections_desc rotated based on current selection
    let min_selection = selections_desc.iter().min().ok_or_else(|| {
        KakMessage(
            "Internal error".to_string(),
            Some("No selections in selections_desc".to_string()),
        )
    })?;
    // Need to rotate selections by primary selection's position in the list
    match selections_desc.iter().position(|p| p == min_selection) {
        Some(i) => {
            selections.rotate_right(i);
        }
        None => {
            return Err(KakMessage(
                "Internal error".to_string(),
                Some(format!(
                    "Primary selection {} not found in selection_desc list ({:#?})",
                    min_selection, selections_desc
                )),
            ))
        }
    }

    selections
        .into_iter()
        .zip(selections_desc.into_iter())
        .map(|(content, desc)| Ok(SelectionWithDesc { content, desc }))
        .collect::<Result<Vec<_>, _>>()
}

/// # Errors
///
/// Will return `Err` if command fifo could not be opened, read from, or written to
pub fn set_selections<'a, I, S: 'a + ?Sized>(selections: I) -> Result<(), KakMessage>
where
    I: IntoIterator<Item = &'a S>,
    S: AsRef<str> + fmt::Display,
{
    let mut f = open_command_fifo()?;
    write!(f, "reg '\"'")?;
    for i in selections {
        write!(f, " '{}'", i.as_ref().replace('\'', "''"))?;
    }
    write!(f, "; exec R;")?;
    f.flush()?;
    Ok(())
}

/// # Errors
///
/// Will return `Err` if command fifo could not be opened, read from, or written to
pub fn set_selections_desc<'a, I>(selections: I) -> Result<(), KakMessage>
where
    I: IntoIterator<Item = &'a SelectionDesc>,
{
    let mut f = open_command_fifo()?;
    write!(f, "select")?;
    for i in selections {
        write!(f, " {}", i)?;
    }
    write!(f, ";")?;
    f.flush()?;
    Ok(())
}

/// # Errors
///
/// Will return `Err` if command fifo could not be opened, read from, or written to
pub fn send_message(msg: &KakMessage) -> Result<(), Box<dyn std::error::Error>> {
    let msg_str = msg.0.replace('\'', "''");
    {
        let mut f =
            open_command_fifo().map_err(|e| format!("Could not open command fifo: {:?}", e))?;

        write!(f, "echo '{}';", msg_str)?;
        write!(f, "echo -debug '{}';", msg_str)?;

        if let Some(debug_msg_str) = &msg.1 {
            write!(f, "echo -debug '{}';", debug_msg_str.replace('\'', "''"))?;
        }
        f.flush()?;
    }
    Ok(())
}

/// # Errors
///
/// Will return `Err` if command fifo could not be opened or written to
pub fn exec(cmd: &str) -> Result<(), KakMessage> {
    let mut f = open_command_fifo()?;

    write!(f, "{}", cmd)?;
    f.flush().map_err(Into::into)
}

/// # Errors
///
/// Will return `Err` if command fifo could not be opened or written to
pub fn response(msg: &str) -> Result<Vec<String>, KakMessage> {
    exec(&format!(
        "echo -quoting shell -to-file {} -- {msg}",
        get_var("kak_response_fifo")?
    ))?;

    let selections = shellwords::split(&fs::read_to_string(&get_var("kak_response_fifo")?)?)?;

    Ok(selections)
}

// pub fn response_iter(msg: &str) -> Result<ShellWordsIterator, KakMessage> {
//     exec(&format!(
//         "echo -quoting shell -to-file {} -- {msg}",
//         get_var("kak_response_fifo")?
//     ))?;

//     Ok(shellwords::split_iter(&fs::read_to_string(&get_var(
//         "kak_response_fifo",
//     )?)?))
// }

/// # Errors
///
/// Will return `Err` if command fifo could not be opened
pub fn open_command_fifo() -> Result<BufWriter<File>, KakMessage> {
    OpenOptions::new()
        .write(true)
        .append(true)
        .open(&get_var("kak_command_fifo")?)
        .map(BufWriter::new)
        .map_err(Into::into)
}

#[cfg(test)]
mod test {
    use super::*;
    const SD: SelectionDesc = SelectionDesc {
        left: AnchorPosition { row: 18, col: 9 },
        right: AnchorPosition { row: 10, col: 1 },
    };
    #[test]
    fn test_anchor_position() {
        // Check parsing
        assert_eq!(SelectionDesc::from_str("18.9,10.1").unwrap(), SD);
        // Check if multiple parsed ones match
        assert_eq!(
            SelectionDesc::from_str("18.9,10.1").unwrap(),
            SelectionDesc::from_str("18.9,10.1").unwrap()
        );
    }

    #[test]
    fn test_sort() {
        // Check if sorting works
        assert_eq!(SD.sort(), SelectionDesc::from_str("10.1,18.9").unwrap());
        assert_eq!(SD.sort(), SD.sort().sort());
    }

    #[test]
    fn test_contains() {
        assert!(SD.contains(&SD));
        assert!(SD.contains(&SelectionDesc::from_str("17.9,10.1").unwrap()));
        assert!(SD.contains(&SelectionDesc::from_str("18.8,10.1").unwrap()));
        assert!(SD.contains(&SelectionDesc::from_str("18.9,11.1").unwrap()));
        assert!(SD.contains(&SelectionDesc::from_str("18.9,10.2").unwrap()));
        assert!(SD.contains(&SelectionDesc::from_str("10.1,17.9").unwrap()));
        assert!(SD.contains(&SelectionDesc::from_str("10.1,18.8").unwrap()));
        assert!(SD.contains(&SelectionDesc::from_str("11.1,18.9").unwrap()));
        assert!(SD.contains(&SelectionDesc::from_str("10.2,18.9").unwrap()));
        assert!(!SD.contains(&SelectionDesc::from_str("19.9,10.1").unwrap()));
        assert!(!SD.contains(&SelectionDesc::from_str("18.10,10.1").unwrap()));
        assert!(!SD.contains(&SelectionDesc::from_str("18.9,9.1").unwrap()));
        assert!(!SD.contains(&SelectionDesc::from_str("18.9,10.0").unwrap()));
        assert!(!SD.contains(&SelectionDesc::from_str("10.1,19.9").unwrap()));
        assert!(!SD.contains(&SelectionDesc::from_str("10.1,18.10").unwrap()));
        assert!(!SD.contains(&SelectionDesc::from_str("9.1,18.9").unwrap()));
        assert!(!SD.contains(&SelectionDesc::from_str("10.0,18.9").unwrap()));
    }
}
