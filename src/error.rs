use std::error;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Alignment(usize),
    Json(serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Alignment(alignment) => write!(f, "Unsupported legend alignment {alignment}"),
            Self::Json(error) => write!(f, "Invalid KLE JSON: {error}"),
        }
    }
}

impl error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}
