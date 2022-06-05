use crate::KakError;
use std::{fmt, str::FromStr};

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
    type Err = KakError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (left, right) = s
            .split_once(',')
            .ok_or_else(|| KakError::Parse(format!("Could not parse as position: {}", s)))?;

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
    type Err = KakError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (left, right) = s
            .split_once('.')
            .ok_or_else(|| KakError::Parse(format!("Could not parse as position: {}", s)))?;
        Ok(Self {
            row: usize::from_str(left)?,
            col: usize::from_str(right)?,
        })
    }
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
