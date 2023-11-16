pub mod usb;
pub mod io;
pub mod utils;

use std::sync::{Arc, Mutex};

use pyo3::prelude::*;

use usb::PyUsbDeviceInfo;
use io::{PyIO, PyIOBitMode};
use neovi_mic_rs::mic::{find_neovi_mics, NeoVIMIC};

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
            self.0.lock().unwrap().get_io_device().unwrap().to_owned(),
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


/// A Python module implemented in Rust.
#[pymodule]
fn neovi_mic(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyNeoVIMIC>()?;
    m.add_class::<PyUsbDeviceInfo>()?;
    m.add_class::<PyIO>()?;
    m.add_class::<PyIOBitMode>()?;
    m.add_function(wrap_pyfunction!(find, m)?)?;
    Ok(())
}
