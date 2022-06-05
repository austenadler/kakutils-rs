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
