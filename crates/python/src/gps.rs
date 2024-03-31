use std::sync::{Arc, Mutex};
use pyo3::prelude::*;

use neovi_mic_rs::nmea::types::GPSInfo;
use neovi_mic_rs::gps::GPSDevice;

use crate::utils::create_python_object;

create_python_object!(PyGPSInfo, "GPSInfo", GPSInfo);
#[pymethods]
impl PyGPSInfo {
    #[new]
    fn py_new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn __str__(&self) -> String {
        /*
        let (latitude, ns) = match &self.0.lock().unwrap().latitude {
            Some((dms, dir)) => format!("{} {} {}", dms.),
            None => "None".to_string(),
        };
        */
        format!("GPSInfo TODO!").to_string()
    }

    fn __repr__(&self) -> String {
        let description = self.__str__();
        format!("<GPSInfo {description}>").to_string()
    }

    #[getter]
    fn current_time(&self, py: Python) -> PyResult<PyObject> {
        let current_time = self.0.lock().unwrap().current_time.unwrap();
        Ok(current_time.to_object(py))
    }
}

impl PyGPSInfo {
    pub fn from(gps_info: &GPSInfo) -> Self {
        Self {
            0: Arc::new(Mutex::new(gps_info.to_owned())),
        }
    }
}



create_python_object!(PyGPSDevice, "GPSDevice", GPSDevice);
#[pymethods]
impl PyGPSDevice {
    #[new]
    fn py_new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn __str__(&self) -> String {
        let description = "GPSDevice TODO!"; // = &self.0.lock().unwrap().to_string();
        format!("{description}").to_string()
    }

    fn __repr__(&self) -> String {
        let description = self.__str__();
        format!("<GPSDevice {description}>").to_string()
    }

    #[getter]
    fn get_info(&self, py: Python) -> PyResult<PyGPSInfo> {
        let info = self.0.lock().unwrap().get_info();
        Ok(PyGPSInfo::from(&info))
    }

    #[getter]
    fn has_lock(&self) -> PyResult<bool> {
        Ok(self.0.lock().unwrap().has_lock())
    }
}

/*
impl PyGPSDevice {
    pub fn from(gps_device: GPSDevice) -> Self {
        Self {
            0: Arc::new(Mutex::new(gps_device.to_owned())),
        }
    }
}
*/