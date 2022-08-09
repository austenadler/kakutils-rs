use crate::KakError;
use core::fmt::{Display, Formatter};
use std::{
    cmp::{max, min},
    fmt,
    str::FromStr,
};

pub type Selection = String;

#[derive(PartialEq, Eq, Debug)]
pub enum MaybeSplit<T> {
    Nothing,
    Just(T),
    JustTwo(T, T),
}

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

/// A selection desc that spans only one row
///
/// This type is required when doing operations that involve multiple lines, but logic cannot exist to see if, for example, 2 selection descs with row:1 col:1-10 and row:2 col:0-1 is adjacent
#[derive(Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Debug)]
pub struct RowSelectionDesc {
    pub row: usize,
    pub left_col: usize,
    pub right_col: usize,
}

// impl RowSelectionDesc {
//     pub fn
// }

impl TryFrom<SelectionDesc> for RowSelectionDesc {
    type Error = KakError;
    fn try_from(sd: SelectionDesc) -> Result<Self, Self::Error> {
        if sd.left.row == sd.right.row {
            Ok(Self {
                row: sd.left.row,
                left_col: sd.left.col,
                right_col: sd.right.col,
            })
        } else {
            Err(KakError::MultiRowSelectionNotSupported)
        }
    }
}

impl Into<SelectionDesc> for RowSelectionDesc {
    fn into(self) -> SelectionDesc {
        SelectionDesc {
            left: AnchorPosition {
                row: self.row,
                col: self.left_col,
            },
            right: AnchorPosition {
                row: self.row,
                col: self.right_col,
            },
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq, Debug)]
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
    pub fn contains<ISD>(&self, other: ISD) -> bool
    where
        ISD: Into<Self>,
    {
        // Cursor and anchor can be flipped. Set a.0 to be leftmost cursor
        let (a, b) = (self.sort(), other.into().sort());

        b.left >= a.left && b.right <= a.right
    }

    #[must_use]
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        // Set a and b to the leftmost and rightmost selection
        let (a, b) = (min(self, other).sort(), max(self, other).sort());

        if a.contains(&b.left)
            || a.contains(&b.right)
            || b.contains(&a.left)
            || b.contains(&a.right)
        {
            // Some(Self {})
            None
        } else {
            None
        }
    }

    #[must_use]
    pub fn partial_union(&self, other: &Self) -> Option<Self> {
        // Set a and b to the leftmost and rightmost selection
        let (a, b) = (min(self, other).sort(), max(self, other).sort());
        eprintln!("Partial union args: a: {a:#?}, b: {b:#?}");

        eprintln!(
            "Checking if {} contains {}: {}",
            a,
            b.left,
            a.contains(&b.left)
        );
        eprintln!(
            "Checking if {} contains {}: {}",
            b,
            a.right,
            b.contains(&a.right)
        );

        // Either the left side of b is contained b a, or
        // This will not work when the right side of a is the end of line and the left side of b is beginning of line
        // This is because selection descs do not know when a selection desc is at the end of a line
        if a.contains(&b.left) || b.contains(&a.right)
        // If b's left is one col off from a's right
        // || (a.right.row == b_left.row && a.right.col == b.left.col.saturating_sub(1))
        // Or b's right is
        // || (a.left.row == b_left.row && a.left.col == b.right.col.saturating_sub(1))
        {
            Some(SelectionDesc {
                left: min(a.left, b.left),
                right: max(a.right, b.right),
            })
        } else {
            None
        }
    }

    #[must_use]
    pub fn subtract(&self, b: &Self) -> MaybeSplit<Self> {
        let sorted_b = b.sort();

        match (sorted_b.contains(&self.left), sorted_b.contains(&self.right), self.contains(&sorted_b)) {
            (true, true, _) => {
                // self is contained by sorted_b
                MaybeSplit::Nothing
            }
            (false, false, false) => {
                // There is no intersection
                // TODO: Why can't I clone myself?
                MaybeSplit::Just(self.clone())
            }
            (false, false, true) => {
                // B is contained and it does not intersect with left or right
                MaybeSplit::JustTwo(
                    Self {
                        left: self.left,
                        right: AnchorPosition {
                            row: sorted_b.left.row,
                            col: sorted_b.left.col.saturating_sub(1),
                        },
                    },
                    Self {
                        left: AnchorPosition {
                            row: sorted_b.right.row,
                            col: sorted_b.right.col.saturating_add(1),
                        },
                        right: self.right,
                    },
                )
            }
            (true, false, _) => {
                // Only self's left is contained
                MaybeSplit::Just(Self {
                    left: AnchorPosition {
                        row: sorted_b.right.row,
                        col: sorted_b.right.col.saturating_add(1),
                    },
                    right: self.right,
                })
            }
            (false, true, _) => {
                // Only self's right is contained
                MaybeSplit::Just(Self {
                    left: self.left,
                    right: AnchorPosition {
                        row: sorted_b.left.row,
                        col: sorted_b.left.col.saturating_sub(1),
                    },
                })
            }
        }
    }
}

impl From<&SelectionDesc> for SelectionDesc {
    fn from(sd: &SelectionDesc) -> Self {
        sd.clone()
    }
}

impl From<&AnchorPosition> for SelectionDesc {
    fn from(ap: &AnchorPosition) -> Self {
        Self {
            left: ap.clone(),
            right: ap.clone(),
        }
    }
}

impl AsRef<SelectionDesc> for SelectionDesc {
    fn as_ref(&self) -> &Self {
        &self
    }
}

impl Display for SelectionDesc {
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

#[derive(PartialOrd, PartialEq, Copy, Clone, Eq, Ord, Debug)]
pub struct AnchorPosition {
    pub row: usize,
    pub col: usize,
}
impl Display for AnchorPosition {
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

impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.kak_escaped())
    }
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

    macro_rules! mixed_test {
        ($left:tt, $op:ident, $right:tt, $expected:expr) => {
            eprintln!("Testing ({}).{}({})", sd!$left, stringify!($op), &sd!$right);
            assert_eq!(sd!$left.$op(&sd!$right), $expected);

            eprintln!("Testing ({}).{}({})", sd!$left, stringify!($op), &sdr!$right);
            assert_eq!(sd!$left.$op(&sdr!$right), $expected);

            eprintln!("Testing ({}).{}({})", sdr!$left, stringify!($op), &sd!$right);
            assert_eq!(sdr!$left.$op(&sd!$right), $expected);

            eprintln!("Testing ({}).{}({})", sdr!$left, stringify!($op), &sdr!$right);
            assert_eq!(sdr!$left.$op(&sdr!$right), $expected);
        }
    }

    const SD: SelectionDesc = SelectionDesc {
        left: AnchorPosition { row: 18, col: 9 },
        right: AnchorPosition { row: 10, col: 1 },
    };
    #[test]
    fn test_anchor_position() {
        // Check parsing
        assert_eq!(sd!(18, 9, 10, 1), SD);
        // Check if multiple parsed ones match
        assert_eq!(sd!(18, 9, 10, 1), sd!(18, 9, 10, 1));
    }

    #[test]
    fn test_sort() {
        assert_eq!(SD.sort(), sd!(10, 1, 18, 9));
        assert_eq!(SD.sort(), SD.sort().sort());
        assert_eq!(sd!(10, 1, 18, 9).sort(), sd!(10, 1, 18, 9));
        assert_eq!(sdr!(10, 1, 18, 9).sort(), sd!(10, 1, 18, 9));
    }

    #[test]
    fn test_contains() {
        assert!(SD.contains(&SD));
        assert!(SD.contains(&sd!(17, 9, 10, 1)));
        assert!(SD.contains(&sd!(18, 8, 10, 1)));
        assert!(SD.contains(&sd!(18, 9, 11, 1)));
        assert!(SD.contains(&sd!(18, 9, 10, 2)));
        assert!(SD.contains(&sd!(10, 1, 17, 9)));
        assert!(SD.contains(&sd!(10, 1, 18, 8)));
        assert!(SD.contains(&sd!(11, 1, 18, 9)));
        assert!(SD.contains(&sd!(10, 2, 18, 9)));
        assert!(!SD.contains(&sd!(19, 9, 10, 1)));
        assert!(!SD.contains(&sd!(18, 10, 10, 1)));
        assert!(!SD.contains(&sd!(18, 9, 9, 1)));
        assert!(!SD.contains(&sd!(18, 9, 10, 0)));
        assert!(!SD.contains(&sd!(10, 1, 19, 9)));
        assert!(!SD.contains(&sd!(10, 1, 18, 10)));
        assert!(!SD.contains(&sd!(9, 1, 18, 9)));
        assert!(!SD.contains(&sd!(10, 0, 18, 9)));

        assert!(SD.contains(&sdr!(17, 9, 10, 1)));
        assert!(SD.contains(&sdr!(18, 8, 10, 1)));
        assert!(SD.contains(&sdr!(18, 9, 11, 1)));
        assert!(SD.contains(&sdr!(18, 9, 10, 2)));
        assert!(SD.contains(&sdr!(10, 1, 17, 9)));
        assert!(SD.contains(&sdr!(10, 1, 18, 8)));
        assert!(SD.contains(&sdr!(11, 1, 18, 9)));
        assert!(SD.contains(&sdr!(10, 2, 18, 9)));
        assert!(!SD.contains(&sdr!(19, 9, 10, 1)));
        assert!(!SD.contains(&sdr!(18, 10, 10, 1)));
        assert!(!SD.contains(&sdr!(18, 9, 9, 1)));
        assert!(!SD.contains(&sdr!(18, 9, 10, 0)));
        assert!(!SD.contains(&sdr!(10, 1, 19, 9)));
        assert!(!SD.contains(&sdr!(10, 1, 18, 10)));
        assert!(!SD.contains(&sdr!(9, 1, 18, 9)));
        assert!(!SD.contains(&sdr!(10, 0, 18, 9)));
    }

    #[test]
    fn test_partial_union() {
        // Testing a+b

        //    01234567
        // a:  ^_^
        // b: ^____^
        mixed_test!((1, 3), partial_union, (0, 5), Some(sd!(0, 5)));

        //    01234567
        // a: ^__^
        // b: ^____^
        mixed_test!((0, 3), partial_union, (0, 5), Some(sd!(0, 5)));

        //    01234567
        // a:  ^___^
        // b:  ^___^
        mixed_test!((1, 5), partial_union, (1, 5), Some(sd!(1, 5)));

        //    01234567
        // a: ^_____^
        // b: ^____^
        mixed_test!((0, 6), partial_union, (0, 5), Some(sd!(0, 6)));

        //    01234567
        // a:  ^____^
        // b: ^____^
        mixed_test!((1, 6), partial_union, (0, 5), Some(sd!(0, 6)));

        //    01234567
        // a: ^____^
        // b:  ^____^
        mixed_test!((0, 5), partial_union, (1, 6), Some(sd!(0, 6)));

        //    01234567
        // a: ^______^
        // b:  ^____^
        mixed_test!((0, 7), partial_union, (1, 6), Some(sd!(0, 7)));

        //    01234567
        // a:    ^
        // b: ^____^
        mixed_test!((3, 3), partial_union, (0, 5), Some(sd!(0, 5)));

        //    01234567
        // a: ^
        // b: ^____^
        mixed_test!((0, 0), partial_union, (0, 5), Some(sd!(0, 5)));

        //    01234567
        // a: ^
        // b:  ^____^
        mixed_test!((0, 0), partial_union, (1, 6), Some(sd!(0, 6)));

        //    01234567
        // a:      ^
        // b: ^____^
        mixed_test!((5, 5), partial_union, (0, 5), Some(sd!(0, 5)));

        //    01234567
        // a:       ^
        // b: ^____^
        mixed_test!((6, 6), partial_union, (0, 5), Some(sd!(0, 6)));

        //    01234567
        // a:        ^
        // b: ^____^
        mixed_test!((7, 7), partial_union, (0, 5), None);

        //    01234567
        // a: ^
        // b:   ^____^
        mixed_test!((0, 0), partial_union, (2, 7), None);
    }

    #[test]
    fn test_subtract() {
        // Testing a-b

        //    01234567
        // a:  ^_^
        // b: ^____^
        assert_eq!(sd!(1, 3).subtract(&sd!(0, 5)), MaybeSplit::Nothing);
        assert_eq!(sd!(1, 3).subtract(&sdr!(0, 5)), MaybeSplit::Nothing);

        //    01234567
        // a: ^__^
        // b: ^____^
        assert_eq!(sd!(0, 3).subtract(&sd!(0, 5)), MaybeSplit::Nothing);
        assert_eq!(sd!(0, 3).subtract(&sdr!(0, 5)), MaybeSplit::Nothing);

        //    01234567
        // a:  ^___^
        // b:  ^___^
        assert_eq!(sd!(1, 5).subtract(&sd!(1, 5)), MaybeSplit::Nothing);
        assert_eq!(sd!(1, 5).subtract(&sdr!(1, 5)), MaybeSplit::Nothing);

        //    01234567
        // a: ^_____^
        // b: ^____^
        assert_eq!(sd!(0, 6).subtract(&sd!(0, 5)), MaybeSplit::Just(sd!(6, 6)));
        assert_eq!(sd!(0, 6).subtract(&sdr!(0, 5)), MaybeSplit::Just(sd!(6, 6)));

        //    01234567
        // a:  ^____^
        // b: ^____^
        assert_eq!(sd!(1, 6).subtract(&sd!(0, 5)), MaybeSplit::Just(sd!(6, 6)));
        assert_eq!(sd!(1, 6).subtract(&sdr!(0, 5)), MaybeSplit::Just(sd!(6, 6)));

        //    01234567
        // a: ^____^
        // b:  ^____^
        assert_eq!(sd!(0, 5).subtract(&sd!(1, 6)), MaybeSplit::Just(sd!(0, 0)));
        assert_eq!(sd!(0, 5).subtract(&sdr!(1, 6)), MaybeSplit::Just(sd!(0, 0)));

        //    01234567
        // a: ^______^
        // b:  ^____^
        assert_eq!(sd! (0, 7).subtract(&sd!(1, 6)), MaybeSplit::JustTwo(sd!(0, 0), sd!(7, 7)) );
        assert_eq!(sd! (0, 7).subtract(&sdr!(1, 6)), MaybeSplit::JustTwo(sd!(0, 0), sd!(7, 7)) );

        //    01234567
        // a:    ^
        // b: ^____^
        assert_eq!(sd!(3, 3).subtract(&sd!(0, 5)), MaybeSplit::Nothing);
        assert_eq!(sd!(3, 3).subtract(&sdr!(0, 5)), MaybeSplit::Nothing);

        //    01234567
        // a: ^
        // b: ^____^
        assert_eq!(sd!(0, 0).subtract(&sd!(0, 5)), MaybeSplit::Nothing);
        assert_eq!(sd!(0, 0).subtract(&sdr!(0, 5)), MaybeSplit::Nothing);

        //    01234567
        // a: ^
        // b:  ^____^
        assert_eq!(sd!(0, 0).subtract(&sd!(1, 6)), MaybeSplit::Just(sd!(0, 0)));
        assert_eq!(sd!(0, 0).subtract(&sdr!(1, 6)), MaybeSplit::Just(sd!(0, 0)));

        //    01234567
        // a:      ^
        // b: ^____^
        assert_eq!(sd!(5, 5).subtract(&sd!(0, 5)), MaybeSplit::Nothing);
        assert_eq!(sd!(5, 5).subtract(&sdr!(0, 5)), MaybeSplit::Nothing);

        //    01234567
        // a:       ^
        // b: ^____^
        assert_eq!(sd!(6, 6).subtract(&sd!(0, 5)), MaybeSplit::Just(sd!(6, 6)));
        assert_eq!(sd!(6, 6).subtract(&sdr!(0, 5)), MaybeSplit::Just(sd!(6, 6)));
    }
}
