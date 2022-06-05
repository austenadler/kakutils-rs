use std::num::ParseIntError;

#[derive(Debug)]
pub enum KakError {
    /// A required environment variable was not set
    EnvVarNotSet(String),
    /// An environment variable was not parsable in unicode
    EnvVarUnicode(String),
    /// There was an error parsing a response from kak
    Parse(String),
    /// There was an error with a response kak gave
    KakResponse(String),
    /// IO Error
    Io(std::io::Error),
    /// Not yet implemented
    NotImplemented(&'static str),
    /// Custom error string
    Custom(String),
}

impl KakError {
    pub fn details(&self) -> String {
        match self {
            Self::EnvVarNotSet(e) => e.clone(),
            Self::EnvVarUnicode(e) => e.clone(),
            Self::Parse(e) => e.clone(),
            Self::KakResponse(e) => e.clone(),
            Self::Io(e) => format!("{e:?}"),
            Self::NotImplemented(e) => e.to_string(),
            Self::Custom(s) => s.clone(),
        }
    }
}

impl ToString for KakError {
    fn to_string(&self) -> String {
        format!(
            "Error: {}",
            match self {
                Self::EnvVarNotSet(_) => "env var not set",
                Self::EnvVarUnicode(_) => "env var not unicode",
                Self::Parse(_) => "Could not parse",
                Self::KakResponse(_) => "Invalid kak response",
                Self::Io(_) => "IO error",
                Self::NotImplemented(_) => "Not Implemented",
                Self::Custom(s) => &s,
            }
        )
    }
}

impl From<std::io::Error> for KakError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<shellwords::MismatchedQuotes> for KakError {
    fn from(e: shellwords::MismatchedQuotes) -> Self {
        Self::Parse(format!("Shell could not be parsed: {e:?}"))
    }
}

impl From<ParseIntError> for KakError {
    fn from(e: ParseIntError) -> Self {
        Self::Parse(format!("Could not parse as integer: {e:?}"))
    }
}
