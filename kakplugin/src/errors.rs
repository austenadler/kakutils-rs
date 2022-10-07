use crate::Register;
use std::{fmt, fmt::Display, num::ParseIntError};

#[derive(Debug)]
pub enum KakError {
    /// A required environment variable was not set
    EnvVarNotSet(String),
    /// An environment variable was not parsable in unicode
    EnvVarUnicode(String),
    /// There was an error parsing a response from kak
    Parse(String),
    /// The string could not be converted into UTF8
    Utf8Error(std::string::FromUtf8Error),
    /// There was an error with a response kak gave
    KakResponse(String),
    /// IO Error
    Io(std::io::Error),
    /// Not yet implemented
    NotImplemented(&'static str),
    /// Custom error string
    Custom(String),
    /// Custom static error string
    CustomStatic(&'static str),
    /// The selections/selections_desc list passed was empty
    SetEmptySelections,
    /// The register register has no content
    EmptyRegister(Register),
}

impl std::error::Error for KakError {}

impl KakError {
    pub fn details(&self) -> String {
        match self {
            Self::EnvVarNotSet(e) => e.clone(),
            Self::EnvVarUnicode(e) => e.clone(),
            Self::Parse(e) => e.clone(),
            Self::Utf8Error(e) => e.to_string(),
            Self::KakResponse(e) => e.clone(),
            Self::Io(e) => format!("{e:?}"),
            Self::NotImplemented(e) => e.to_string(),
            Self::Custom(s) => s.clone(),
            Self::CustomStatic(s) => s.to_string(),
            Self::SetEmptySelections => {
                String::from("Attempted to set selections/selections_desc to empty list")
            }
            Self::EmptyRegister(r) => {
                format!("Empty register: {r}")
            }
        }
    }
}

impl Display for KakError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: ")?;
        match self {
            Self::EnvVarNotSet(_) => write!(f, "env var not set"),
            Self::EnvVarUnicode(_) => write!(f, "env var not unicode"),
            Self::Parse(_) => write!(f, "Could not parse"),
            Self::Utf8Error(_) => write!(f, "The string is not valid UTF-8"),
            Self::KakResponse(_) => write!(f, "Invalid kak response"),
            Self::Io(_) => write!(f, "IO error"),
            Self::NotImplemented(_) => write!(f, "Not Implemented"),
            Self::Custom(s) => write!(f, "{}", s),
            Self::CustomStatic(s) => write!(f, "{}", s),
            Self::SetEmptySelections => write!(
                f,
                "Attempted to set selections/selections_desc to empty list"
            ),
            Self::EmptyRegister(r) => write!(f, "Register {r} has no content"),
        }
    }
}

impl From<std::convert::Infallible> for KakError {
    fn from(_e: std::convert::Infallible) -> Self {
        Self::NotImplemented("Infallible error encountered")
    }
}

impl From<std::io::Error> for KakError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<shell_words::ParseError> for KakError {
    fn from(e: shell_words::ParseError) -> Self {
        Self::Parse(format!("Shell could not be parsed: {e:?}"))
    }
}

impl From<ParseIntError> for KakError {
    fn from(e: ParseIntError) -> Self {
        Self::Parse(format!("Could not parse as integer: {e:?}"))
    }
}

impl From<std::string::FromUtf8Error> for KakError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::Utf8Error(e)
    }
}
