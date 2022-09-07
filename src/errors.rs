use std::num::ParseIntError;

#[derive(Debug)]
pub struct KakMessage(pub String, pub Option<String>);

impl From<std::io::Error> for KakMessage {
    fn from(err: std::io::Error) -> Self {
        Self(
            "Error writing to fifo".to_string(),
            Some(format!("{}", err)),
        )
    }
}

impl From<String> for KakMessage {
    fn from(err: String) -> Self {
        Self(err, None)
    }
}

impl From<&str> for KakMessage {
    fn from(err: &str) -> Self {
        Self(err.to_string(), None)
    }
}

impl From<kakplugin::ParseError> for KakMessage {
    fn from(err: kakplugin::ParseError) -> Self {
        Self("Corrupt kak response".to_string(), Some(err.to_string()))
    }
}

impl From<ParseIntError> for KakMessage {
    fn from(err: ParseIntError) -> Self {
        Self(format!("Could not parse int: {}", err), None)
    }
}
