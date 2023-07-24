use std::fmt;

use serialport;


#[derive(Debug, Clone)]
pub enum Error {
    InvalidDevice(String),
    SerialError(serialport::Error),

}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::InvalidDevice(s) => write!(f, "Invalid Device: {:#?}", s),
            Self::SerialError(s) => write!(f, "Serial Error: {:#?}", s),
        }
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::InvalidDevice(value.to_string())
    }
}

/// Generic crate Result object
pub type Result<T> = std::result::Result<T, Error>;
