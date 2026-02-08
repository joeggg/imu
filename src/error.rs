use std::{error::Error as StdError, fmt::Display, io};

#[derive(Debug)]
pub enum Error {
    StreamDecodeFailure(String),
    InvalidData(String),
    Unknown(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StreamDecodeFailure(msg) => write!(f, "{}", msg),
            Self::InvalidData(msg) => write!(f, "{}", msg),
            Self::Unknown(msg) => write!(f, "{}", msg),
        }
    }
}

impl StdError for Error {}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Unknown(format!("{:?}", value))
    }
}
