use std::sync::{Arc, Mutex};

use neovi_mic_rs::io;
use neovi_mic_rs::io::IO;
use neovi_mic_rs::mic::{UsbDeviceInfo, NeoVIMIC};
use pyo3::prelude::*;

/// Create a thread safe python object (struct).
/// 
/// Note: use `self.0.lock().unwrap()` to obtain the 
/// inner object since its wrapped in an Arc<Mutex>.
/// 
/// # Arguments
/// * `$name:ident` - Name of the struct visible in Rust.
/// 
/// * `$python_name` - Name of the object visible to Python.
/// 
/// * `$inner_name:ty` - name of the transparent inner struct we are trying to wrap.
/// 
/// # Example
/// ```no_run
/// create_python_object!(PyMyStruct, "MyStruct", MyStruct)
/// ```
macro_rules! create_python_object {
    ($name:ident, $python_name:literal, $inner_name:ty) => {
        #[pyclass(name=$python_name)]
        #[derive(Debug, Default, Clone)]
        #[repr(transparent)]
        pub struct $name(pub Arc<Mutex<$inner_name>>);

        // Arc is only Send if T is Send so lets mark it as safe here
        unsafe impl Send for $name {}
    };
}

create_python_object!(PyNeoVIMIC, "NeoVIMIC", NeoVIMIC);
#[pymethods]
impl PyNeoVIMIC {
    #[new]
    fn py_new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn __str__(&self) -> String {
        let serial = self.0.lock().unwrap().get_serial_number();
        format!("NeoVI MIC2 {serial}").to_string()
    }

    fn __repr__(&self) -> String {
        let description = self.__str__();
        format!("<NeoVI MIC2 {description}").to_string()
    }

    fn get_serial_number(&self) -> PyResult<String> {
        Ok(self.0.lock().unwrap().get_serial_number())
    }

    fn has_gps(&self) -> PyResult<bool> {
        Ok(self.0.lock().unwrap().has_gps())
    }

    #[getter]
    fn ftdi(&self) -> PyResult<PyUsbDeviceInfo> {
        Ok(PyUsbDeviceInfo::from(
            self.0.lock().unwrap().get_ftdi_device().unwrap(),
        ))
    }

    #[getter]
    fn io(&self) -> PyResult<PyIO> {
        Ok(PyIO::from(
            self.0.lock().unwrap().get_io_device().unwrap(),
        ))
    }
}

impl PyNeoVIMIC {
    pub fn from(neovi_mic: NeoVIMIC) -> Self {
        Self {
            0: Arc::new(Mutex::new(neovi_mic)),
        }
    }
}

create_python_object!(PyUsbDeviceInfo, "UsbDeviceInfo", UsbDeviceInfo);
#[pymethods]
impl PyUsbDeviceInfo {
    #[new]
    fn py_new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn __str__(&self) -> String {
        let serial = match &self.0.lock().unwrap().serial_number {
            Some(s) => s.clone(),
            None => "None".to_string(),
        };
        format!("NeoVI MIC2 {serial}").to_string()
    }

    fn __repr__(&self) -> String {
        let description = self.__str__();
        format!("<NeoDevice {description}").to_string()
    }

    #[getter]
    fn vendor_id(&self) -> PyResult<u16> {
        Ok(self.0.lock().unwrap().vendor_id)
    }

    #[getter]
    fn product_id(&self) -> PyResult<u16> {
        Ok(self.0.lock().unwrap().product_id)
    }

    #[getter]
    fn bus_number(&self) -> PyResult<u8> {
        Ok(self.0.lock().unwrap().bus_number)
    }

    #[getter]
    fn address(&self) -> PyResult<u8> {
        Ok(self.0.lock().unwrap().address)
    }

    /* TODO
    #[getter]
    fn device_type(&self) -> PyResult<u32> {
        Ok(self.0.lock().unwrap().device_type.into())
    }
    */
}

impl PyUsbDeviceInfo {
    pub fn from(usb_device_info: &UsbDeviceInfo) -> Self {
        Self {
            0: Arc::new(Mutex::new(usb_device_info.to_owned())),
        }
    }
}

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
            PyIOBitMode::None as u8,
            io::IOBitMode::None.bits() as u8
        );
        assert_eq!(
            PyIOBitMode::Buzzer as u8,
            io::IOBitMode::Buzzer.bits() as u8
        );
        assert_eq!(
            PyIOBitMode::Button as u8,
            io::IOBitMode::Button.bits() as u8
        );
        assert_eq!(
            PyIOBitMode::GPSLed as u8,
            io::IOBitMode::GPSLed.bits() as u8
        );
        assert_eq!(
            PyIOBitMode::CBUS3 as u8,
            io::IOBitMode::CBUS3.bits() as u8
        );
        assert_eq!(
            PyIOBitMode::BuzzerMask as u8,
            io::IOBitMode::BuzzerMask.bits() as u8
        );
        assert_eq!(
            PyIOBitMode::ButtonMask as u8,
            io::IOBitMode::ButtonMask.bits() as u8
        );
        assert_eq!(
            PyIOBitMode::GPSLedMask as u8,
            io::IOBitMode::GPSLedMask.bits() as u8
        );
        assert_eq!(
            PyIOBitMode::CBUS3Mask as u8,
            io::IOBitMode::CBUS3Mask.bits() as u8
        );
        assert_eq!(
            PyIOBitMode::DefaultMask as u8,
            io::IOBitMode::DefaultMask.bits() as u8
        );
    }
}
