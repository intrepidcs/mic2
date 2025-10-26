use pyo3::{exceptions::PyValueError, prelude::*};
use std::sync::{Arc, Mutex};

use mic2::nmea::types::{GPSInfo, GPSSatInfo, GpsNavigationStatus, GPSDMS};

use crate::utils::create_python_object;

#[pyclass]
#[derive(Clone, Copy)]
pub enum PyGpsNavigationStatus {
    NoFix = GpsNavigationStatus::NoFix as isize,
    DeadReckoningOnly = GpsNavigationStatus::DeadReckoningOnly as isize,
    StandAlone2D = GpsNavigationStatus::StandAlone2D as isize,
    StandAlone3D = GpsNavigationStatus::StandAlone3D as isize,
    Differential2D = GpsNavigationStatus::Differential2D as isize,
    Differential3D = GpsNavigationStatus::Differential3D as isize,
    CombinedRKGPSDeadReckoning = GpsNavigationStatus::CombinedRKGPSDeadReckoning as isize,
    TimeOnly = GpsNavigationStatus::TimeOnly as isize,
}

impl PyGpsNavigationStatus {
    pub fn from(gps_dms: GpsNavigationStatus) -> Self {
        match gps_dms {
            GpsNavigationStatus::NoFix => PyGpsNavigationStatus::NoFix,
            GpsNavigationStatus::DeadReckoningOnly => PyGpsNavigationStatus::DeadReckoningOnly,
            GpsNavigationStatus::StandAlone2D => PyGpsNavigationStatus::StandAlone2D,
            GpsNavigationStatus::StandAlone3D => PyGpsNavigationStatus::StandAlone3D,
            GpsNavigationStatus::Differential2D => PyGpsNavigationStatus::Differential2D,
            GpsNavigationStatus::Differential3D => PyGpsNavigationStatus::Differential3D,
            GpsNavigationStatus::CombinedRKGPSDeadReckoning => {
                PyGpsNavigationStatus::CombinedRKGPSDeadReckoning
            }
            GpsNavigationStatus::TimeOnly => PyGpsNavigationStatus::TimeOnly,
        }
    }
}

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
        let gps_info = &self.0.lock().unwrap();
        format!("{gps_info:?}")
    }

    fn __repr__(&self) -> String {
        let description = self.__str__();
        format!("<GPSInfo {description}>").to_string()
    }

    #[getter]
    fn current_time(&self, py: Python) -> PyResult<Py<PyAny>> {
        use pyo3::types::PyDateTime;
        match self.0.lock().unwrap().current_time {
            Some(current_time) => {
                let py_datetime = PyDateTime::from_timestamp(
                    py,
                    current_time.and_utc().timestamp() as f64,
                    None,
                )?;
                Ok(py_datetime.into())
            },
            None => Err(PyValueError::new_err("current_time not valid")),
        }
    }

    fn latitude(&self) -> PyResult<(PyGPSDMS, char)> {
        match self.0.lock().unwrap().latitude {
            Some((dms, dir)) => Ok((PyGPSDMS::from(dms), dir)),
            None => Err(PyValueError::new_err("latitude not valid")),
        }
    }

    fn longitude(&self) -> PyResult<(PyGPSDMS, char)> {
        match self.0.lock().unwrap().longitude {
            Some((dms, dir)) => Ok((PyGPSDMS::from(dms), dir)),
            None => Err(PyValueError::new_err("longitude not valid")),
        }
    }

    fn nav_stat(&self) -> PyResult<PyGpsNavigationStatus> {
        match self.0.lock().unwrap().nav_stat {
            Some(nav_stat) => Ok(PyGpsNavigationStatus::from(nav_stat)),
            None => Err(PyValueError::new_err("nav_stat not valid")),
        }
    }

    fn satellites(&self) -> PyResult<Vec<PyGPSSatInfo>> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .satellites
            .clone()
            .into_iter()
            .map(PyGPSSatInfo::from)
            .collect::<Vec<PyGPSSatInfo>>())
    }

    #[getter]
    fn altitude(&self) -> PyResult<f64> {
        match self.0.lock().unwrap().altitude {
            Some(altitude) => Ok(altitude),
            None => Err(PyValueError::new_err("altitude not valid")),
        }
    }

    #[getter]
    fn h_acc(&self) -> PyResult<f64> {
        match self.0.lock().unwrap().h_acc {
            Some(h_acc) => Ok(h_acc),
            None => Err(PyValueError::new_err("h_acc not valid")),
        }
    }

    #[getter]
    fn v_acc(&self) -> PyResult<f64> {
        match self.0.lock().unwrap().v_acc {
            Some(v_acc) => Ok(v_acc),
            None => Err(PyValueError::new_err("v_acc not valid")),
        }
    }

    #[getter]
    fn sog_kmh(&self) -> PyResult<f64> {
        match self.0.lock().unwrap().v_acc {
            Some(v_acc) => Ok(v_acc),
            None => Err(PyValueError::new_err("v_acc not valid")),
        }
    }

    #[getter]
    fn cog(&self) -> PyResult<f64> {
        match self.0.lock().unwrap().cog {
            Some(cog) => Ok(cog),
            None => Err(PyValueError::new_err("cog not valid")),
        }
    }

    #[getter]
    fn vvel(&self) -> PyResult<f64> {
        match self.0.lock().unwrap().vvel {
            Some(vvel) => Ok(vvel),
            None => Err(PyValueError::new_err("vvel not valid")),
        }
    }

    #[getter]
    fn age_c(&self) -> PyResult<f64> {
        match self.0.lock().unwrap().age_c {
            Some(age_c) => Ok(age_c),
            None => Err(PyValueError::new_err("age_c not valid")),
        }
    }

    #[getter]
    fn hdop(&self) -> PyResult<f64> {
        match self.0.lock().unwrap().hdop {
            Some(hdop) => Ok(hdop),
            None => Err(PyValueError::new_err("hdop not valid")),
        }
    }

    #[getter]
    fn vdop(&self) -> PyResult<f64> {
        match self.0.lock().unwrap().vdop {
            Some(vdop) => Ok(vdop),
            None => Err(PyValueError::new_err("vdop not valid")),
        }
    }

    #[getter]
    fn tdop(&self) -> PyResult<f64> {
        match self.0.lock().unwrap().tdop {
            Some(tdop) => Ok(tdop),
            None => Err(PyValueError::new_err("tdop not valid")),
        }
    }

    #[getter]
    fn clock_bias(&self) -> PyResult<f64> {
        match self.0.lock().unwrap().clock_bias {
            Some(clock_bias) => Ok(clock_bias),
            None => Err(PyValueError::new_err("clock_bias not valid")),
        }
    }

    #[getter]
    fn clock_drift(&self) -> PyResult<f64> {
        match self.0.lock().unwrap().clock_drift {
            Some(clock_drift) => Ok(clock_drift),
            None => Err(PyValueError::new_err("clock_drift not valid")),
        }
    }

    #[getter]
    fn timepulse_granularity(&self) -> PyResult<f64> {
        match self.0.lock().unwrap().timepulse_granularity {
            Some(timepulse_granularity) => Ok(timepulse_granularity),
            None => Err(PyValueError::new_err("timepulse_granularity not valid")),
        }
    }
}

impl PyGPSInfo {
    pub fn from(gps_info: &GPSInfo) -> Self {
        Self(Arc::new(Mutex::new(gps_info.to_owned())))
    }
}

create_python_object!(PyGPSDMS, "GPSDMS", GPSDMS);
#[pymethods]
impl PyGPSDMS {
    #[new]
    fn py_new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn __str__(&self) -> String {
        self.0.lock().unwrap().to_string()
    }

    fn __repr__(&self) -> String {
        let description = self.__str__();
        format!("<PyGPSDMS {description}>").to_string()
    }

    #[getter]
    fn degrees(&self) -> u16 {
        self.0.lock().unwrap().degrees
    }

    #[getter]
    fn minutes(&self) -> u8 {
        self.0.lock().unwrap().minutes
    }

    #[getter]
    fn seconds(&self) -> u8 {
        self.0.lock().unwrap().seconds
    }
}

impl PyGPSDMS {
    pub fn from(gps_dms: GPSDMS) -> Self {
        Self(Arc::new(Mutex::new(gps_dms.to_owned())))
    }
}

create_python_object!(PyGPSSatInfo, "GPSSatInfo", GPSSatInfo);
#[pymethods]
impl PyGPSSatInfo {
    #[new]
    fn py_new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn __str__(&self) -> String {
        self.0.lock().unwrap().to_string()
    }

    fn __repr__(&self) -> String {
        let description = self.__str__();
        format!("<PyGPSSatInfo {description}>").to_string()
    }

    #[getter]
    fn prn(&self) -> u16 {
        self.0.lock().unwrap().prn
    }

    #[getter]
    fn used(&self) -> bool {
        self.0.lock().unwrap().used
    }

    #[getter]
    fn azimuth(&self) -> PyResult<u16> {
        match self.0.lock().unwrap().azimuth {
            Some(azimuth) => Ok(azimuth),
            None => Err(PyValueError::new_err("azimuth not valid")),
        }
    }

    #[getter]
    fn elevation(&self) -> PyResult<u16> {
        match self.0.lock().unwrap().elevation {
            Some(elevation) => Ok(elevation),
            None => Err(PyValueError::new_err("elevation not valid")),
        }
    }

    #[getter]
    fn snr(&self) -> PyResult<u8> {
        match self.0.lock().unwrap().snr {
            Some(snr) => Ok(snr),
            None => Err(PyValueError::new_err("snr not valid")),
        }
    }

    #[getter]
    fn lock_time(&self) -> u8 {
        self.0.lock().unwrap().lock_time
    }
}

impl PyGPSSatInfo {
    pub fn from(gps_sat_info: GPSSatInfo) -> Self {
        Self(Arc::new(Mutex::new(gps_sat_info.to_owned())))
    }
}
