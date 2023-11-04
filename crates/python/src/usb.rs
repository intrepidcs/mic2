use std::sync::{Mutex, Arc};

use neovi_mic_rs::mic::UsbDeviceInfo;
use pyo3::prelude::*;

use crate::utils::create_python_object;

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
