use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ExtractionError {
    TooSmall,
    ParseError,
    UnknownType,
    MissingDataPoint,
    Unknown(String),
}

impl Display for ExtractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooSmall => write!(f, "Got too small a message"),
            Self::ParseError => write!(f, "Failed to parse string as float"),
            Self::UnknownType => write!(f, "Unknown data point type"),
            Self::MissingDataPoint => write!(f, "Missing a data point"),
            Self::Unknown(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for ExtractionError {}
