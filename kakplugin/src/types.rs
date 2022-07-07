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

impl AsRef<SelectionDesc> for SelectionDesc {
    fn as_ref(&self) -> &Self {
        &self
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

#[derive(Debug, PartialEq, Eq)]
pub enum Register {
    Numeric0,
    Numeric1,
    Numeric2,
    Numeric3,
    Numeric4,
    Numeric5,
    Numeric6,
    Numeric7,
    Numeric8,
    Numeric9,

    UppercaseA,
    UppercaseB,
    UppercaseC,
    UppercaseD,
    UppercaseE,
    UppercaseF,
    UppercaseG,
    UppercaseH,
    UppercaseI,
    UppercaseJ,
    UppercaseK,
    UppercaseL,
    UppercaseM,
    UppercaseN,
    UppercaseO,
    UppercaseP,
    UppercaseQ,
    UppercaseR,
    UppercaseS,
    UppercaseT,
    UppercaseU,
    UppercaseV,
    UppercaseW,
    UppercaseX,
    UppercaseY,
    UppercaseZ,

    LowercaseA,
    LowercaseB,
    LowercaseC,
    LowercaseD,
    LowercaseE,
    LowercaseF,
    LowercaseG,
    LowercaseH,
    LowercaseI,
    LowercaseJ,
    LowercaseK,
    LowercaseL,
    LowercaseM,
    LowercaseN,
    LowercaseO,
    LowercaseP,
    LowercaseQ,
    LowercaseR,
    LowercaseS,
    LowercaseT,
    LowercaseU,
    LowercaseV,
    LowercaseW,
    LowercaseX,
    LowercaseY,
    LowercaseZ,

    Dquote,
    Slash,
    Arobase,
    Caret,
    Pipe,
    Percent,
    Dot,
    Hash,
    Underscore,
    Colon,
}

impl Register {
    pub fn kak_expanded(&self) -> &'static str {
        match self {
            Self::Dquote => "dquote",
            Self::Slash => "slash",
            Self::Arobase => "arobase",
            Self::Caret => "caret",
            Self::Pipe => "pipe",
            Self::Percent => "percent",
            Self::Dot => "dot",
            Self::Hash => "hash",
            Self::Underscore => "underscore",
            Self::Colon => "colon",

            // Everything else is the same
            _ => self.kak_escaped(),
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Self::Numeric0 => '0',
            Self::Numeric1 => '1',
            Self::Numeric2 => '2',
            Self::Numeric3 => '3',
            Self::Numeric4 => '4',
            Self::Numeric5 => '5',
            Self::Numeric6 => '6',
            Self::Numeric7 => '7',
            Self::Numeric8 => '8',
            Self::Numeric9 => '9',

            Self::UppercaseA => 'A',
            Self::UppercaseB => 'B',
            Self::UppercaseC => 'C',
            Self::UppercaseD => 'D',
            Self::UppercaseE => 'E',
            Self::UppercaseF => 'F',
            Self::UppercaseG => 'G',
            Self::UppercaseH => 'H',
            Self::UppercaseI => 'I',
            Self::UppercaseJ => 'J',
            Self::UppercaseK => 'K',
            Self::UppercaseL => 'L',
            Self::UppercaseM => 'M',
            Self::UppercaseN => 'N',
            Self::UppercaseO => 'O',
            Self::UppercaseP => 'P',
            Self::UppercaseQ => 'Q',
            Self::UppercaseR => 'R',
            Self::UppercaseS => 'S',
            Self::UppercaseT => 'T',
            Self::UppercaseU => 'U',
            Self::UppercaseV => 'V',
            Self::UppercaseW => 'W',
            Self::UppercaseX => 'X',
            Self::UppercaseY => 'Y',
            Self::UppercaseZ => 'Z',

            Self::LowercaseA => 'a',
            Self::LowercaseB => 'b',
            Self::LowercaseC => 'c',
            Self::LowercaseD => 'd',
            Self::LowercaseE => 'e',
            Self::LowercaseF => 'f',
            Self::LowercaseG => 'g',
            Self::LowercaseH => 'h',
            Self::LowercaseI => 'i',
            Self::LowercaseJ => 'j',
            Self::LowercaseK => 'k',
            Self::LowercaseL => 'l',
            Self::LowercaseM => 'm',
            Self::LowercaseN => 'n',
            Self::LowercaseO => 'o',
            Self::LowercaseP => 'p',
            Self::LowercaseQ => 'q',
            Self::LowercaseR => 'r',
            Self::LowercaseS => 's',
            Self::LowercaseT => 't',
            Self::LowercaseU => 'u',
            Self::LowercaseV => 'v',
            Self::LowercaseW => 'w',
            Self::LowercaseX => 'x',
            Self::LowercaseY => 'y',
            Self::LowercaseZ => 'z',

            Self::Dquote => '"',
            Self::Slash => '/',
            Self::Arobase => '@',
            Self::Caret => '^',
            Self::Pipe => '|',
            Self::Percent => '%',
            Self::Dot => '.',
            Self::Hash => '#',
            Self::Underscore => '_',
            Self::Colon => ':',
        }
    }

    pub fn kak_escaped(&self) -> &'static str {
        match self {
            Self::Numeric0 => "0",
            Self::Numeric1 => "1",
            Self::Numeric2 => "2",
            Self::Numeric3 => "3",
            Self::Numeric4 => "4",
            Self::Numeric5 => "5",
            Self::Numeric6 => "6",
            Self::Numeric7 => "7",
            Self::Numeric8 => "8",
            Self::Numeric9 => "9",

            Self::UppercaseA => "A",
            Self::UppercaseB => "B",
            Self::UppercaseC => "C",
            Self::UppercaseD => "D",
            Self::UppercaseE => "E",
            Self::UppercaseF => "F",
            Self::UppercaseG => "G",
            Self::UppercaseH => "H",
            Self::UppercaseI => "I",
            Self::UppercaseJ => "J",
            Self::UppercaseK => "K",
            Self::UppercaseL => "L",
            Self::UppercaseM => "M",
            Self::UppercaseN => "N",
            Self::UppercaseO => "O",
            Self::UppercaseP => "P",
            Self::UppercaseQ => "Q",
            Self::UppercaseR => "R",
            Self::UppercaseS => "S",
            Self::UppercaseT => "T",
            Self::UppercaseU => "U",
            Self::UppercaseV => "V",
            Self::UppercaseW => "W",
            Self::UppercaseX => "X",
            Self::UppercaseY => "Y",
            Self::UppercaseZ => "Z",

            Self::LowercaseA => "a",
            Self::LowercaseB => "b",
            Self::LowercaseC => "c",
            Self::LowercaseD => "d",
            Self::LowercaseE => "e",
            Self::LowercaseF => "f",
            Self::LowercaseG => "g",
            Self::LowercaseH => "h",
            Self::LowercaseI => "i",
            Self::LowercaseJ => "j",
            Self::LowercaseK => "k",
            Self::LowercaseL => "l",
            Self::LowercaseM => "m",
            Self::LowercaseN => "n",
            Self::LowercaseO => "o",
            Self::LowercaseP => "p",
            Self::LowercaseQ => "q",
            Self::LowercaseR => "r",
            Self::LowercaseS => "s",
            Self::LowercaseT => "t",
            Self::LowercaseU => "u",
            Self::LowercaseV => "v",
            Self::LowercaseW => "w",
            Self::LowercaseX => "x",
            Self::LowercaseY => "y",
            Self::LowercaseZ => "z",

            Self::Dquote => "\\\"",
            Self::Slash => "/",
            Self::Arobase => "@",
            Self::Caret => "^",
            Self::Pipe => "|",
            Self::Percent => "%",
            Self::Dot => ".",
            Self::Hash => "#",
            Self::Underscore => "_",
            Self::Colon => ":",
        }
    }
}

impl AsRef<Register> for Register {
    fn as_ref(&self) -> &Self {
        &self
    }
}

impl FromStr for Register {
    type Err = KakError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Self::Numeric0),
            "1" => Ok(Self::Numeric1),
            "2" => Ok(Self::Numeric2),
            "3" => Ok(Self::Numeric3),
            "4" => Ok(Self::Numeric4),
            "5" => Ok(Self::Numeric5),
            "6" => Ok(Self::Numeric6),
            "7" => Ok(Self::Numeric7),
            "8" => Ok(Self::Numeric8),
            "9" => Ok(Self::Numeric9),

            "A" => Ok(Self::UppercaseA),
            "B" => Ok(Self::UppercaseB),
            "C" => Ok(Self::UppercaseC),
            "D" => Ok(Self::UppercaseD),
            "E" => Ok(Self::UppercaseE),
            "F" => Ok(Self::UppercaseF),
            "G" => Ok(Self::UppercaseG),
            "H" => Ok(Self::UppercaseH),
            "I" => Ok(Self::UppercaseI),
            "J" => Ok(Self::UppercaseJ),
            "K" => Ok(Self::UppercaseK),
            "L" => Ok(Self::UppercaseL),
            "M" => Ok(Self::UppercaseM),
            "N" => Ok(Self::UppercaseN),
            "O" => Ok(Self::UppercaseO),
            "P" => Ok(Self::UppercaseP),
            "Q" => Ok(Self::UppercaseQ),
            "R" => Ok(Self::UppercaseR),
            "S" => Ok(Self::UppercaseS),
            "T" => Ok(Self::UppercaseT),
            "U" => Ok(Self::UppercaseU),
            "V" => Ok(Self::UppercaseV),
            "W" => Ok(Self::UppercaseW),
            "X" => Ok(Self::UppercaseX),
            "Y" => Ok(Self::UppercaseY),
            "Z" => Ok(Self::UppercaseZ),

            "a" => Ok(Self::LowercaseA),
            "b" => Ok(Self::LowercaseB),
            "c" => Ok(Self::LowercaseC),
            "d" => Ok(Self::LowercaseD),
            "e" => Ok(Self::LowercaseE),
            "f" => Ok(Self::LowercaseF),
            "g" => Ok(Self::LowercaseG),
            "h" => Ok(Self::LowercaseH),
            "i" => Ok(Self::LowercaseI),
            "j" => Ok(Self::LowercaseJ),
            "k" => Ok(Self::LowercaseK),
            "l" => Ok(Self::LowercaseL),
            "m" => Ok(Self::LowercaseM),
            "n" => Ok(Self::LowercaseN),
            "o" => Ok(Self::LowercaseO),
            "p" => Ok(Self::LowercaseP),
            "q" => Ok(Self::LowercaseQ),
            "r" => Ok(Self::LowercaseR),
            "s" => Ok(Self::LowercaseS),
            "t" => Ok(Self::LowercaseT),
            "u" => Ok(Self::LowercaseU),
            "v" => Ok(Self::LowercaseV),
            "w" => Ok(Self::LowercaseW),
            "x" => Ok(Self::LowercaseX),
            "y" => Ok(Self::LowercaseY),
            "z" => Ok(Self::LowercaseZ),

            "\"" | "dquote" => Ok(Self::Dquote),
            "/" | "slash" => Ok(Self::Slash),
            "@" | "arobase" => Ok(Self::Arobase),
            "^" | "caret" => Ok(Self::Caret),
            "|" | "pipe" => Ok(Self::Pipe),
            "%" | "percent" => Ok(Self::Percent),
            "." | "dot" => Ok(Self::Dot),
            "#" | "hash" => Ok(Self::Hash),
            "_" | "underscore" => Ok(Self::Underscore),
            ":" | "colon" => Ok(Self::Colon),

            _ => Err(KakError::Parse(format!(
                "Register '{s}' could not be parsed"
            ))),
        }
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
