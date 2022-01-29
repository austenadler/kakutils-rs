#[derive(Debug)]
pub struct KakMessage(pub String, pub Option<String>);

impl From<std::io::Error> for KakMessage {
    fn from(err: std::io::Error) -> Self {
        Self(
            "Error writing to fifo".to_string(),
            Some(format!("{:?}", err)),
        )
    }
}

impl From<String> for KakMessage {
    fn from(err: String) -> Self {
        Self(err, None)
    }
}

impl From<shellwords::MismatchedQuotes> for KakMessage {
    fn from(err: shellwords::MismatchedQuotes) -> Self {
        Self("Corrupt kak response".to_string(), Some(err.to_string()))
    }
}
