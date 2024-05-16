pub mod gps;
pub mod usb;
pub mod utils;

use std::sync::{Arc, Mutex};

use pyo3::prelude::*;

use ::mic2::{find_neovi_mics, NeoVIMIC};
use usb::PyUsbDeviceInfo;

use gps::{PyGPSDMS, PyGPSInfo, PyGPSSatInfo};

use crate::utils::create_python_object;

#[pyfunction]
fn find() -> PyResult<Vec<PyNeoVIMIC>> {
    let devices = find_neovi_mics()
        .unwrap()
        .into_iter()
        .map(|x| PyNeoVIMIC::from(x))
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
        match self.0.lock().unwrap().get_usb_io_info() {
            Some(info) => Some(PyUsbDeviceInfo::from(info)),
            None => None,
        }
    }

    fn get_usb_audio_info(&self) -> Option<PyUsbDeviceInfo> {
        match self.0.lock().unwrap().get_usb_audio_info() {
            Some(info) => Some(PyUsbDeviceInfo::from(info)),
            None => None,
        }
    }

    fn get_usb_gps_info(&self) -> Option<PyUsbDeviceInfo> {
        match self.0.lock().unwrap().get_usb_gps_info() {
            Some(info) => Some(PyUsbDeviceInfo::from(info)),
            None => None,
        }
    }

    fn get_usb_extra_info(&self) -> Vec<PyUsbDeviceInfo> {
        self.0
            .lock()
            .unwrap()
            .get_usb_extra_info()
            .iter()
            .map(|info| PyUsbDeviceInfo::from(info))
            .collect()
    }

    fn io_open(&self) -> PyResult<()> {
        Ok(self.0.lock().unwrap().io_open().unwrap())
    }

    fn io_close(&self) -> PyResult<()> {
        Ok(self.0.lock().unwrap().io_close().unwrap())
    }

    fn io_is_open(&self) -> PyResult<bool> {
        Ok(self.0.lock().unwrap().io_is_open().unwrap())
    }

    fn io_buzzer_enable(&self, enabled: bool) -> PyResult<()> {
        Ok(self.0.lock().unwrap().io_buzzer_enable(enabled).unwrap())
    }

    fn io_buzzer_is_enabled(&self) -> PyResult<bool> {
        Ok(self.0.lock().unwrap().io_buzzer_is_enabled().unwrap())
    }

    fn io_gpsled_enable(&self, enabled: bool) -> PyResult<()> {
        Ok(self.0.lock().unwrap().io_gpsled_enable(enabled).unwrap())
    }

    fn io_gpsled_is_enabled(&self) -> PyResult<bool> {
        Ok(self.0.lock().unwrap().io_gpsled_is_enabled().unwrap())
    }

    fn io_button_is_pressed(&self) -> PyResult<bool> {
        Ok(self.0.lock().unwrap().io_button_is_pressed().unwrap())
    }

    fn audio_start(&self, sample_rate: u32) -> PyResult<()> {
        Ok(self.0.lock().unwrap().audio_start(sample_rate).unwrap())
    }

    fn audio_stop(&self) -> PyResult<()> {
        Ok(self.0.lock().unwrap().audio_stop().unwrap())
    }

    fn audio_save(&self, fname: String) -> PyResult<()> {
        Ok(self.0.lock().unwrap().audio_save(fname).unwrap())
    }

    fn gps_open(&self) -> PyResult<bool> {
        Ok(self.0.lock().unwrap().gps_open().unwrap())
    }

    fn gps_close(&self) -> PyResult<()> {
        Ok(self.0.lock().unwrap().gps_close().unwrap())
    }

    fn gps_is_open(&self) -> PyResult<bool> {
        Ok(self.0.lock().unwrap().gps_is_open().unwrap())
    }

    fn gps_has_lock(&self) -> PyResult<bool> {
        Ok(self.0.lock().unwrap().gps_has_lock().unwrap())
    }

    fn gps_info(&self) -> PyResult<PyGPSInfo> {
        Ok(PyGPSInfo::from(&self.0.lock().unwrap().gps_info().unwrap()))
    }
}

impl PyNeoVIMIC {
    pub fn from(neovi_mic: NeoVIMIC) -> Self {
        Self {
            0: Arc::new(Mutex::new(neovi_mic)),
        }
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
