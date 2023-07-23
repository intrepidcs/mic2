
pub enum Error {
    InvalidDevice(String),

}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::InvalidDevice(s) => write!(f, "Invalid Device: {:#?}", s),
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

#[derive(Debug, Clone, default)]
pub struct NeoVIMIC {
    /// Serial number of the neoVI MIC, MCxxxx
    serial_number: String,
    /// Serial port identifier for the GPS functionality. Some devices
    /// don't have GPS so this is optional.
    gps_serial_port: Option<String>,
    /// Audio Capture device name attached to the neoVI MIC.
    audio_name: String,
    /// Index of the FTDI device attached to the neoVI MIC. This is for IO
    /// control of the device (button, speaker, and LEDs).
    ftdi_index: u32,
};

impl NeoVIMIC {
    pub fn new(index: u32) -> Result<Self> {
        todo!();
        Ok(
            Self {
            ..Default::default()
            }
        )
    }

    pub fn from_serial_number(serial_number: impl Into<String>) -> Result<Self> {
        todo!();
        Ok(
            Self {
                serial_number,
                ..Default::default()
        })
    }
}