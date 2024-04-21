use std::sync::{Arc, Mutex};

use neovi_mic_rs::mic;

#[repr(transparent)]
pub struct NeoVIMIC {
    inner: Arc<Mutex<mic::NeoVIMIC>>,
}

impl NeoVIMIC {
    pub fn from(neovi_mic: mic::NeoVIMIC) -> Self {
        Self {
            inner: Arc::new(Mutex::new(neovi_mic)),
        }
    }
}

#[repr(u32)]
pub enum NeoVIMICErrType {
    // Function was successful.
    NeoVIMICErrTypeSuccess,
    // Function failed.
    NeoVIMICErrTypeFailure,
    // Function failed due to invalid parameter.
    NeoVIMICErrTypeInvalidParameter,
}

/// Find all neovi MIC2s.
///
/// @param devices    Pointer to an array of NeoVIMIC structs. Initialize to nullptr. Must call mic2_free() when done.
/// @param length     Length of devices. Must point to valid memory
/// 
/// @return           NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
#[no_mangle]
pub extern "C" fn mic2_find(devices: *mut *mut NeoVIMIC, length: *mut u32) -> NeoVIMICErrType {
    if !devices.is_null() || length.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }

    let mut found_devices = match mic::find_neovi_mics() {
        Ok(d) => d,
        Err(_e) => return NeoVIMICErrType::NeoVIMICErrTypeFailure,
    }.into_iter()
    .map(|x| NeoVIMIC::from(x))
    .collect::<Vec<NeoVIMIC>>();

    unsafe {
        *length = found_devices.len() as u32;
        *devices = found_devices.as_mut_ptr();
        std::mem::forget(found_devices);
    };
    NeoVIMICErrType::NeoVIMICErrTypeSuccess
}

/// Open the IO interface on the device. 
///
/// @param device    Pointer to aNeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
/// 
/// @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
#[no_mangle]
pub extern "C" fn mic2_io_open(device: *mut NeoVIMIC) -> NeoVIMICErrType {
    if !device.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }
    let neovi_mic = unsafe { &*device }.inner.lock().unwrap();
    match neovi_mic.io_open() {
        Ok(_) => NeoVIMICErrType::NeoVIMICErrTypeSuccess,
        Err(_e) => NeoVIMICErrType::NeoVIMICErrTypeFailure,
    }
}

/// Close the IO interface on the device. 
///
/// @param device    Pointer to aNeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
/// 
/// @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
#[no_mangle]
pub extern "C" fn mic2_io_close(device: *mut NeoVIMIC) -> NeoVIMICErrType {
    if !device.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }
    let neovi_mic = unsafe { &*device }.inner.lock().unwrap();
    match neovi_mic.io_close() {
        Ok(_) => NeoVIMICErrType::NeoVIMICErrTypeSuccess,
        Err(_e) => NeoVIMICErrType::NeoVIMICErrTypeFailure,
    }
}

/// Check if the IO interface on the device is open.
///
/// @param device    Pointer to aNeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
/// @param is_open   Pointer to a bool. Set to true if open, false if not. Returns NeoVIMICErrTypeInvalidParameter if nullptr
/// 
/// @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
#[no_mangle]
pub extern "C" fn mic2_io_is_open(device: *mut NeoVIMIC, is_open: *mut bool) -> NeoVIMICErrType {
    if !device.is_null() || is_open.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }
    unsafe { *is_open = false };
    
    let neovi_mic = unsafe { &*device }.inner.lock().unwrap();
    match neovi_mic.io_is_open() {
        Ok(b) => {
            unsafe { *is_open = b };
            NeoVIMICErrType::NeoVIMICErrTypeSuccess
        },
        Err(_e) => NeoVIMICErrType::NeoVIMICErrTypeFailure,
    }
}

/// Enable the IO Buzzer on the device.
///
/// @param device    Pointer to aNeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
/// @param enable   Set to true to enable, false if not.
/// 
/// @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
#[no_mangle]
pub extern "C" fn mic2_io_buzzer_enable(device: *mut NeoVIMIC, enable: bool) -> NeoVIMICErrType {
    if !device.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }
    let neovi_mic = unsafe { &*device }.inner.lock().unwrap();
    match neovi_mic.io_buzzer_enable(enable) {
        Ok(_) => NeoVIMICErrType::NeoVIMICErrTypeSuccess,
        Err(_e) => NeoVIMICErrType::NeoVIMICErrTypeFailure,
    }
}

/// Check if the IO Buzzer on the device is enabled.
///
/// @param device    Pointer to aNeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
/// @param is_enabled   Pointer to a bool. Set to true if enabled, false if not. Returns NeoVIMICErrTypeInvalidParameter if nullptr
/// 
/// @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
#[no_mangle]
pub extern "C" fn mic2_io_buzzer_is_enabled(device: *mut NeoVIMIC, is_enabled: *mut bool) -> NeoVIMICErrType {
    if !device.is_null() || is_enabled.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }
    unsafe { *is_enabled = false };
    
    let neovi_mic = unsafe { &*device }.inner.lock().unwrap();
    match neovi_mic.io_buzzer_is_enabled() {
        Ok(b) => {
            unsafe { *is_enabled = b };
            NeoVIMICErrType::NeoVIMICErrTypeSuccess
        },
        Err(_e) => NeoVIMICErrType::NeoVIMICErrTypeFailure,
    }
}

/// Free the NeoVIMIC object. This must be called when finished otherwise a memory leak will occur.
///
/// @param device    Pointer to aNeoVIMIC structs. Okay to pass a nullptr.
/// 
/// @return          None
#[no_mangle]
pub unsafe extern "C" fn mic2_free(device: *mut NeoVIMIC) -> () {
    if device.is_null() {
        return;
    }
    std::mem::drop(Box::from_raw(device));
}
