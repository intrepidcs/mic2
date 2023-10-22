use std::sync::{Arc, Mutex};

use pyo3::prelude::*;
use neovi_mic_rs::mic;

macro_rules! define_basic_py_object {
    ($name:ident, $inner_name:ty) => {
        #[pyclass]
        #[derive(Debug)]
        #[repr(transparent)]
        pub struct $name(pub Arc<Mutex<$inner_name>>);

        // Arc is only Send if T is Send so lets mark it as safe here
        unsafe impl Send for $name {}

        #[pymethods]
        impl $name {
            #[new]
            fn py_new() -> Self {
                Self::new()
            }
        }
    };
}

macro_rules! define_basic_py_object_no_new {
    ($name:ident, $inner_name:ty) => {
        #[pyclass]
        #[derive(Debug, Default, Clone)]
        #[repr(transparent)]
        pub struct $name(pub Arc<Mutex<$inner_name>>);

        // Arc is only Send if T is Send so lets mark it as safe here
        unsafe impl Send for $name {}
    };
}

define_basic_py_object_no_new!(NeoVIMIC, mic::NeoVIMIC);
#[pymethods]
impl NeoVIMIC {
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
        format!("<NeoDevice {description}").to_string()
    }

    fn get_serial_number(&self) -> PyResult<String> {
        Ok(self.0.lock().unwrap().get_serial_number())
    }

    fn has_gps(&self) -> PyResult<bool> {
        Ok(self.0.lock().unwrap().has_gps())
    }

    fn get_ftdi_device(&self) -> PyResult<UsbDeviceInfo> {
        Ok(
            UsbDeviceInfo::from(
                self.0.lock().unwrap().get_ftdi_device().unwrap()
            )
        )
    }
}

impl NeoVIMIC {
    /* TODO
    fn new() -> Self {
        Self {
            0: Arc::new(Mutex::new(mic::NeoVIMIC { ..Default::default() })),
        }
    }
     */
    pub fn from(neovi_mic: mic::NeoVIMIC) -> Self {
        Self {
            0: Arc::new(Mutex::new(neovi_mic)),
        }
    }
}

define_basic_py_object_no_new!(UsbDeviceInfo, mic::UsbDeviceInfo);
#[pymethods]
impl UsbDeviceInfo {
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

impl UsbDeviceInfo {
    /* TODO
    fn new() -> Self {
        Self {
            0: Arc::new(Mutex::new(mic::UsbDeviceInfo { ..Default::default() })),
        }
    }
     */
    pub fn from(usb_device_info: mic::UsbDeviceInfo) -> Self {
        Self {
            0: Arc::new(Mutex::new(usb_device_info)),
        }
    }
}