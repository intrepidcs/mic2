pub mod gps;
pub mod usb;
pub mod utils;

use std::sync::{Arc, Mutex};

use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
};

use ::mic2::{find_neovi_mics, NeoVIMIC};
use usb::PyUsbDeviceInfo;

use gps::{PyGPSDMS, PyGPSInfo, PyGPSSatInfo};

use crate::utils::create_python_object;

#[pyfunction]
fn find() -> PyResult<Vec<PyNeoVIMIC>> {
    let devices = find_neovi_mics()
        .unwrap()
        .into_iter()
        .map(PyNeoVIMIC::from)
        .collect::<Vec<PyNeoVIMIC>>();
    Ok(devices)
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
        format!("<NeoVI MIC2 {description}>").to_string()
    }

    fn get_serial_number(&self) -> PyResult<String> {
        Ok(self.0.lock().unwrap().get_serial_number())
    }

    fn has_gps(&self) -> PyResult<bool> {
        Ok(self.0.lock().unwrap().has_gps())
    }

    fn get_usb_hub_info(&self) -> PyUsbDeviceInfo {
        PyUsbDeviceInfo::from(self.0.lock().unwrap().get_usb_hub_info())
    }

    fn get_usb_io_info(&self) -> Option<PyUsbDeviceInfo> {
        self.0
            .lock()
            .unwrap()
            .get_usb_io_info()
            .as_ref()
            .map(PyUsbDeviceInfo::from)
    }

    fn get_usb_audio_info(&self) -> Option<PyUsbDeviceInfo> {
        self.0
            .lock()
            .unwrap()
            .get_usb_audio_info()
            .as_ref()
            .map(PyUsbDeviceInfo::from)
    }

    fn get_usb_gps_info(&self) -> Option<PyUsbDeviceInfo> {
        self.0
            .lock()
            .unwrap()
            .get_usb_gps_info()
            .as_ref()
            .map(PyUsbDeviceInfo::from)
    }

    fn get_usb_extra_info(&self) -> Vec<PyUsbDeviceInfo> {
        self.0
            .lock()
            .unwrap()
            .get_usb_extra_info()
            .iter()
            .map(PyUsbDeviceInfo::from)
            .collect()
    }

    fn io_open(&self) -> PyResult<()> {
        match self.0.lock().unwrap().io_open() {
            Ok(v) => Ok(v),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    fn io_close(&self) -> PyResult<()> {
        match self.0.lock().unwrap().io_close() {
            Ok(v) => Ok(v),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    fn io_is_open(&self) -> PyResult<bool> {
        match self.0.lock().unwrap().io_is_open() {
            Ok(v) => Ok(v),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    fn io_buzzer_enable(&self, enabled: bool) -> PyResult<()> {
        match self.0.lock().unwrap().io_buzzer_enable(enabled) {
            Ok(v) => Ok(v),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    fn io_buzzer_is_enabled(&self) -> PyResult<bool> {
        match self.0.lock().unwrap().io_buzzer_is_enabled() {
            Ok(v) => Ok(v),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    fn io_gpsled_enable(&self, enabled: bool) -> PyResult<()> {
        match self.0.lock().unwrap().io_gpsled_enable(enabled) {
            Ok(v) => Ok(v),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    fn io_gpsled_is_enabled(&self) -> PyResult<bool> {
        match self.0.lock().unwrap().io_gpsled_is_enabled() {
            Ok(v) => Ok(v),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    fn io_button_is_pressed(&self) -> PyResult<bool> {
        match self.0.lock().unwrap().io_button_is_pressed() {
            Ok(v) => Ok(v),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    fn audio_start(&self, sample_rate: u32) -> PyResult<()> {
        self.0.lock().unwrap().audio_start(sample_rate).unwrap();
        Ok(())
    }

    fn audio_stop(&self) -> PyResult<()> {
        self.0.lock().unwrap().audio_stop().unwrap();
        Ok(())
    }

    fn audio_save(&self, fname: String) -> PyResult<()> {
        self.0.lock().unwrap().audio_save(fname).unwrap();
        Ok(())
    }

    fn gps_open(&self) -> PyResult<bool> {
        match self.0.lock().unwrap().gps_open() {
            Ok(v) => Ok(v),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    fn gps_close(&self) -> PyResult<()> {
        match &self.0.lock().unwrap().gps_close() {
            Ok(()) => Ok(()),
            Err(e) => Err(PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn gps_is_open(&self) -> PyResult<bool> {
        match self.0.lock().unwrap().gps_is_open() {
            Ok(v) => Ok(v),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    fn gps_has_lock(&self) -> PyResult<bool> {
        Ok(self.0.lock().unwrap().gps_has_lock().unwrap())
    }

    fn gps_info(&self) -> PyResult<PyGPSInfo> {
        match &self.0.lock().unwrap().gps_info() {
            Ok(gps_info) => Ok(PyGPSInfo::from(gps_info)),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }
}

impl PyNeoVIMIC {
    pub fn from(neovi_mic: NeoVIMIC) -> Self {
        #[allow(clippy::arc_with_non_send_sync)] // FIXME(drebbe): the Arc does nothing
        Self(Arc::new(Mutex::new(neovi_mic)))
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn pymic2(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyNeoVIMIC>()?;
    m.add_class::<PyUsbDeviceInfo>()?;
    m.add_class::<PyGPSInfo>()?;
    m.add_class::<PyGPSDMS>()?;
    m.add_class::<PyGPSSatInfo>()?;
    m.add_function(wrap_pyfunction!(find, m)?)?;
    Ok(())
}
