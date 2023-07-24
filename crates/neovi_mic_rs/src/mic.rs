use crate::types::Result;

#[derive(Debug, Clone, Default)]
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
}

impl NeoVIMIC {
    pub fn new(_index: u32) -> Result<Self> {
        Ok(
            Self {
            ..Default::default()
            }
        )
    }

    pub fn from_serial_number(serial_number: impl Into<String>) -> Result<Self> {
        Ok(
            Self {
                serial_number: serial_number.into(),
                ..Default::default()
        })
    }
}