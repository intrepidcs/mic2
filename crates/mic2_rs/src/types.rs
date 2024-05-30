use serialport;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Error {
    /// Invalid device, typically occurs when the device doesn't have support (ie. GPS)
    InvalidDevice(String),
    SerialError(serialport::Error),
    IOError(std::io::ErrorKind),
    /// Unrecoverable critical error
    CriticalError(String),
    /// Feature is not enabled. See cargo.toml and cargo --features parameters
    NotSupported(String),
}

impl std::error::Error for Error {}
//impl std::io::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::InvalidDevice(s) => write!(f, "Invalid Device: {:#?}", s),
            Self::SerialError(s) => write!(f, "Serial Error: {:#?}", s),
            Self::IOError(e) => write!(f, "IO Error: {:#?}", e),
            Self::CriticalError(s) => write!(f, "Critical Error: {:#?}", s),
            Self::NotSupported(s) => write!(f, "Not Supported: {:#?}", s),
        }
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::InvalidDevice(value.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value.kind())
    }
}

/// Generic crate Result object
pub type Result<T> = std::result::Result<T, Error>;
