use pyo3::prelude::*;

use crate::utils::create_python_object;
use crate::usb::PyUsbDeviceInfo;

use neovi_mic_rs::io;
use neovi_mic_rs::io::IO;

use std::sync::{Mutex, Arc};

#[pyclass(name = "IOBitMode")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PyIOBitMode {
    Buzzer = io::IOBitMode::Buzzer as u8,
    Button = io::IOBitMode::Button as u8,
    GPSLed = io::IOBitMode::GPSLed as u8,
    CBUS3 = io::IOBitMode::CBUS3 as u8,
    BuzzerMask = io::IOBitMode::BuzzerMask as u8,
    ButtonMask = io::IOBitMode::ButtonMask as u8,
    GPSLedMask = io::IOBitMode::GPSLedMask as u8,
    CBUS3Mask = io::IOBitMode::CBUS3Mask as u8,
}

#[pymethods]
impl PyIOBitMode {
    #[getter]
    pub fn value(&self) -> u8 {
        match self {
            Self::Buzzer => Self::Buzzer as u8,
            Self::Button => Self::Button as u8,
            Self::GPSLed => Self::GPSLed as u8,
            Self::CBUS3 => Self::CBUS3 as u8,
            Self::BuzzerMask => Self::BuzzerMask as u8,
            Self::ButtonMask => Self::ButtonMask as u8,
            Self::GPSLedMask => Self::GPSLedMask as u8,
            Self::CBUS3Mask => Self::CBUS3Mask as u8,
        }
    }
}

impl TryFrom<io::IOBitMode> for PyIOBitMode {
    type Error = &'static str;

    fn try_from(value: io::IOBitMode) -> Result<Self, Self::Error> {
        let value = match value {
            io::IOBitMode::Buzzer => Ok(PyIOBitMode::Buzzer),
            io::IOBitMode::Button => Ok(PyIOBitMode::Button),
            io::IOBitMode::GPSLed => Ok(PyIOBitMode::GPSLed),
            io::IOBitMode::CBUS3 => Ok(PyIOBitMode::CBUS3),
            io::IOBitMode::BuzzerMask => Ok(PyIOBitMode::BuzzerMask),
            io::IOBitMode::ButtonMask => Ok(PyIOBitMode::ButtonMask),
            _ => Err("Invalid IODeviceBitMode!"),
        };
        value
    }
}

create_python_object!(PyIO, "IO", IO);
#[pymethods]
impl PyIO {
    #[new]
    fn py_new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn __str__(&self) -> String {
        let serial = match &self.0.lock().unwrap().get_usb_device_info().serial_number {
            Some(s) => s.clone(),
            None => "None".to_string(),
        };
        format!("NeoVI MIC2 {serial}").to_string()
    }

    fn __repr__(&self) -> String {
        let description = self.__str__();
        format!("<NeoVI MIC2 {description}").to_string()
    }

    /// Check if the device is open.
    fn is_open(&self) -> PyResult<bool> {
        Ok(self.0.lock().unwrap().is_open())
    }

    /// Open the device.
    fn open(&self) -> PyResult<()> {
        Ok(self.0.lock().unwrap().open().unwrap())
    }

    /// Open the device.
    fn close(&self) -> PyResult<()> {
        Ok(self.0.lock().unwrap().close().unwrap())
    }

    /// Enable/disable bitbang modes.
    /// bitmask	Bitmask to configure lines. HIGH/ON value configures a line as output.
    /// mode	Bitbang mode: use the values defined in ftdi_mpsse_mode
    ///
    /// CBUS0 = Buzzer
    /// CBUS1 = Button
    /// CBUS2 = GPS LED
    /// CBUS3 = N/C
    ///
    fn set_bitmode_raw(&self, bitmask: u8) -> PyResult<()> {
        Ok(self.0.lock().unwrap().set_bitmode_raw(bitmask).unwrap())
    }

    fn set_bitmode(&self, bitmask: PyIOBitMode) -> PyResult<()> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .set_bitmode(io::IOBitMode::from_bits(bitmask as u8).unwrap())
            .unwrap())
    }

    /// Directly read pin state, circumventing the read buffer. Useful for bitbang mode.
    ///
    /// CBUS0 = Buzzer
    /// CBUS1 = Button
    /// CBUS2 = GPS LED
    /// CBUS3 = N/C
    ///
    fn read_pins_raw(&self) -> PyResult<u8> {
        Ok(self.0.lock().unwrap().read_pins_raw().unwrap())
    }

    fn get_usb_device_info(&self) -> PyResult<PyUsbDeviceInfo> {
        Ok(PyUsbDeviceInfo::from(
            self.0.lock().unwrap().get_usb_device_info(),
        ))
    }
}

impl PyIO {
    pub fn from(io_device: IO) -> Self {
        Self {
            0: Arc::new(Mutex::new(io_device)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_io_device_bit_mode() {
        assert_eq!(
            PyIOBitMode::Buzzer as u8,
            io::IOBitMode::Buzzer as u8
        );
        assert_eq!(
            PyIOBitMode::Button as u8,
            io::IOBitMode::Button as u8
        );
        assert_eq!(
            PyIOBitMode::GPSLed as u8,
            io::IOBitMode::GPSLed as u8
        );
        assert_eq!(
            PyIOBitMode::CBUS3 as u8,
            io::IOBitMode::CBUS3 as u8
        );
        assert_eq!(
            PyIOBitMode::BuzzerMask as u8,
            io::IOBitMode::BuzzerMask as u8
        );
        assert_eq!(
            PyIOBitMode::ButtonMask as u8,
            io::IOBitMode::ButtonMask as u8
        );
        assert_eq!(
            PyIOBitMode::GPSLedMask as u8,
            io::IOBitMode::GPSLedMask as u8
        );
        assert_eq!(
            PyIOBitMode::CBUS3Mask as u8,
            io::IOBitMode::CBUS3Mask as u8
        );
    }
}
