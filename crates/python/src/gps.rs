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

    // TODO: latitude
    // TODO: longitude
    // TODO: nav_stat
    // TODO: satellites

    #[getter]
    fn altitude(&self) -> PyResult<f64> {
        Ok(self.0.lock().unwrap().altitude.unwrap())
    }

    #[getter]
    fn h_acc(&self) -> PyResult<f64> {
        Ok(self.0.lock().unwrap().h_acc.unwrap())
    }

    #[getter]
    fn v_acc(&self) -> PyResult<f64> {
        Ok(self.0.lock().unwrap().v_acc.unwrap())
    }

    #[getter]
    fn sog_kmh(&self) -> PyResult<f64> {
        Ok(self.0.lock().unwrap().sog_kmh.unwrap())
    }

    #[getter]
    fn cog(&self) -> PyResult<f64> {
        Ok(self.0.lock().unwrap().cog.unwrap())
    }

    #[getter]
    fn vvel(&self) -> PyResult<f64> {
        Ok(self.0.lock().unwrap().vvel.unwrap())
    }

    #[getter]
    fn age_c(&self) -> PyResult<f64> {
        Ok(self.0.lock().unwrap().age_c.unwrap())
    }

    #[getter]
    fn hdop(&self) -> PyResult<f64> {
        Ok(self.0.lock().unwrap().hdop.unwrap())
    }

    #[getter]
    fn vdop(&self) -> PyResult<f64> {
        Ok(self.0.lock().unwrap().vdop.unwrap())
    }

    #[getter]
    fn tdop(&self) -> PyResult<f64> {
        Ok(self.0.lock().unwrap().tdop.unwrap())
    }

    #[getter]
    fn clock_bias(&self) -> PyResult<f64> {
        Ok(self.0.lock().unwrap().clock_bias.unwrap())
    }

    #[getter]
    fn clock_drift(&self) -> PyResult<f64> {
        Ok(self.0.lock().unwrap().clock_drift.unwrap())
    }

    #[getter]
    fn timepulse_granularity(&self) -> PyResult<f64> {
        Ok(self.0.lock().unwrap().timepulse_granularity.unwrap())
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
        match self.0.lock().unwrap().get_info() {
            Ok(i) => Ok(PyGPSInfo::from(&i)),
            Err(e) => panic!("{}", e),
        }
    }

    #[getter]
    fn has_lock(&self) -> PyResult<bool> {
        Ok(self.0.lock().unwrap().has_lock().unwrap())
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