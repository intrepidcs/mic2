use std::sync::{Arc, Mutex};

use crate::mic;

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
    /// Function was successful
    NeoVIMICErrTypeSuccess,
    NeoVIMICErrTypeFailure,
    NeoVIMICErrTypeInvalidParameter,
}

#[no_mangle]
pub extern "C" fn mic2_find(devices: *mut *mut NeoVIMIC, length: *mut u32) -> NeoVIMICErrType {
    if !devices.is_null() || length.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }

    let found_devices = match mic::find_neovi_mics() {
        Ok(d) => d,
        Err(_e) => return NeoVIMICErrType::NeoVIMICErrTypeFailure,
    }.into_iter()
    .map(|x| NeoVIMIC::from(x))
    .collect::<Vec<NeoVIMIC>>();

    todo!();
    /*
    let devs: Box<[*mut NeoVIMIC]> = found_devices.try_into().unwrap();

    unsafe {
        *devices = Box::into_raw(devs) 
    };
    */

    NeoVIMICErrType::NeoVIMICErrTypeSuccess
}

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

pub unsafe extern "C" fn mic2_free(device: *mut NeoVIMIC) -> () {
    if device.is_null() {
        return;
    }
    Box::from_raw(device);
}
