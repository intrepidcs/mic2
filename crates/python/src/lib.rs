use pyo3::prelude::*;

use neovi_mic_rs::mic;
pub mod types;
use types::{PyNeoVIMIC, PyIODevice, PyIODeviceBitMode, PyUsbDeviceInfo};

#[pyfunction]
fn find() -> PyResult<Vec<PyNeoVIMIC>> {
    let devices = mic::find_neovi_mics()
        .unwrap()
        .into_iter()
        .map(|x| PyNeoVIMIC::from(x))
        .collect::<Vec<PyNeoVIMIC>>();
    Ok(devices)
}

/// A Python module implemented in Rust.
#[pymodule]
fn neovi_mic(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyNeoVIMIC>()?;
    m.add_class::<PyUsbDeviceInfo>()?;
    m.add_class::<PyIODevice>()?;
    m.add_class::<PyIODeviceBitMode>()?;
    m.add_function(wrap_pyfunction!(find, m)?)?;
    Ok(())
}
