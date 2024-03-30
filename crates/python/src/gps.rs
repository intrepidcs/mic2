use std::sync::{Mutex, Arc};

use neovi_mic_rs::mic::nema::types::GpsInfo;
use pyo3::prelude::*;
use chrono::NaiveDateTime;
use pyo3::ToPyObject;

use crate::utils::create_python_object;

create_python_object!(PyGpsInfo, "GpsInfo", GpsInfo);
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
        format!("GpsInfo TODO!").to_string()
    }

    fn __repr__(&self) -> String {
        let description = self.__str__();
        format!("<GpsInfo {description}").to_string()
    }

    #[getter]
    fn current_time(&self) -> PyResult<u16> {
        self.0.lock().unwrap().current_time.to_object(pyo3::Python::acquire_gil().as_ptr())
        PyDatetime::from();
        Ok(self.0.lock().unwrap().current_time)
    }
}

impl PyUsbDeviceInfo {
    pub fn from(usb_device_info: &UsbDeviceInfo) -> Self {
        Self {
            0: Arc::new(Mutex::new(usb_device_info.to_owned())),
        }
    }
}
