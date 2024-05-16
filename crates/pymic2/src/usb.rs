use std::sync::{Arc, Mutex};

use mic2::UsbDeviceInfo;
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
        let usb_info = self.0.lock().unwrap();
        let serial = match &usb_info.serial_number {
            Some(s) => s.as_str(),
            None => "None",
        };
        format!(
            "{:?} VID: {:#x} PID: {:#x} Bus: {:#x} Addr: {:#x} Serial: {serial}",
            usb_info.device_type,
            usb_info.vendor_id,
            usb_info.product_id,
            usb_info.bus_number,
            usb_info.address
        )
        .to_string()
    }

    fn __repr__(&self) -> String {
        let description = self.__str__();
        format!("<UsbDeviceInfo <{description}>>").to_string()
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

    #[getter]
    fn device_type(&self) -> PyResult<u32> {
        let usb_dev_type = self.0.lock().unwrap();
        Ok(usb_dev_type.device_type as u32)
    }

    #[getter]
    fn device_type_as_str_string(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.0.lock().unwrap().device_type))
    }
}

impl PyUsbDeviceInfo {
    pub fn from(usb_device_info: &UsbDeviceInfo) -> Self {
        Self {
            0: Arc::new(Mutex::new(usb_device_info.to_owned())),
        }
    }
}
