use pyo3::prelude::*;

use neovi_mic_rs::mic;
pub mod types;

#[pyfunction]
fn find() -> PyResult<Vec<types::NeoVIMIC>> {
    let devices = mic::find_neovi_mics()
        .unwrap()
        .into_iter()
        .map(|x| types::NeoVIMIC::from(x))
        .collect::<Vec<types::NeoVIMIC>>();
    Ok(devices)
}

/// A Python module implemented in Rust.
#[pymodule]
fn neovi_mic(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<types::NeoVIMIC>()?;
    m.add_class::<types::UsbDeviceInfo>()?;
    m.add_class::<types::IODevice>()?;
    m.add_class::<types::PyIODeviceBitMode>()?;
    m.add_function(wrap_pyfunction!(find, m)?)?;
    Ok(())
}
