use core::slice;
use neovi_mic_rs::mic;
use std::{
    ffi::{c_void, CString},
    os::raw::c_char,
    sync::{Arc, Mutex},
};

// Version of the API in use. This will allow forward compatibility without having to recompile your application, unless otherwise specified.
pub const MIC2_API_VERSION: u32 = 0x1;

#[derive(Debug, Clone)]
pub struct NeoVIMICHandle {
    inner: Arc<Mutex<mic::NeoVIMIC>>,
}

impl NeoVIMICHandle {
    pub fn from(neovi_mic: mic::NeoVIMIC) -> Self {
        Self {
            inner: Arc::new(Mutex::new(neovi_mic)),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct NeoVIMIC {
    // API version, must be MIC2_API_VERSION
    pub version: u32,
    // Size of the struct, must be sizeof(NeoVIMIC)
    pub size: u32,
    // Serial number of the device. Typically in "MCxxxx" format. Null terminated.
    pub serial_number: [std::ffi::c_char; 16],
    // Handle to the device, should always be valid
    //pub handle: *mut NeoVIMICHandle,
    pub handle: *mut c_void,
}

#[repr(u32)]
pub enum NeoVIMICErrType {
    // Function was successful.
    NeoVIMICErrTypeSuccess = 0,
    // Function failed.
    NeoVIMICErrTypeFailure,
    // Function failed due to invalid parameter.
    NeoVIMICErrTypeInvalidParameter,
    // Requested index is invalid.
    NeoVIMICErrTypeInvalidIndex,
    // Version mismatch, see MIC2_API_VERSION.
    NeoVIMICErrTypeVersionMismatch,
    // Size mismatch. See NeoVIMICHandle::size.
    NeoVIMICErrTypeSizeMismatch,
}

impl From<u32> for NeoVIMICErrType {
    fn from(error_type: u32) -> Self {
        match error_type {
            0 => NeoVIMICErrType::NeoVIMICErrTypeSuccess,
            1 => NeoVIMICErrType::NeoVIMICErrTypeFailure,
            2 => NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter,
            3 => NeoVIMICErrType::NeoVIMICErrTypeInvalidIndex,
            4 => NeoVIMICErrType::NeoVIMICErrTypeVersionMismatch,
            5 => NeoVIMICErrType::NeoVIMICErrTypeSizeMismatch,
            _ => panic!("Unknown NeoVIMICErrType type: {}", error_type),
        }
    }
}

#[no_mangle]
extern "C" fn mic2_error_string(
    error_type: u32,
    buffer: *mut c_char,
    length: *mut u32,
) -> NeoVIMICErrType {
    if buffer.is_null() || length.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }
    // Get the error string
    let error_msg = match NeoVIMICErrType::from(error_type) {
        NeoVIMICErrType::NeoVIMICErrTypeSuccess => "Success",
        NeoVIMICErrType::NeoVIMICErrTypeFailure => "Failure",
        NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter => "Invalid Parameter",
        NeoVIMICErrType::NeoVIMICErrTypeInvalidIndex => "Invalid Index",
        NeoVIMICErrType::NeoVIMICErrTypeVersionMismatch => "Version Mismatch",
        NeoVIMICErrType::NeoVIMICErrTypeSizeMismatch => "Size Mismatch",
    };
    // Convert the buffer to a slice
    let buffer_length = unsafe { *length as usize };
    let buffer_slice = unsafe { slice::from_raw_parts_mut(buffer, buffer_length) };
    let error_msg_cstring = CString::new(error_msg).unwrap();
    // Determine the length of the error string and buffer
    let len = if buffer_length < error_msg_cstring.as_bytes_with_nul().len() {
        // We don't have enough space in the buffer to copy the string
        unsafe { *length = error_msg.len() as u32 };
        buffer_length
    } else {
        error_msg_cstring.as_bytes_with_nul().len()
    };
    // Finally Copy the String to the char buffer
    unsafe {
        buffer_slice[..len].copy_from_slice(slice::from_raw_parts(error_msg_cstring.as_ptr(), len))
    };

    NeoVIMICErrType::NeoVIMICErrTypeSuccess
}
/// Find all neovi MIC2s.
///
/// @param devices    Pointer to an array of NeoVIMIC structs. These need to be allocated by the caller. 
///                   Unused devices should be freed using mic2_free() to avoid memory leaks.
///                   Although this parameter is const, it is still modifed by the function. This is a convinience so all the other 
///                   function calls don't need a const cast.
/// @param length     Length of devices. Must not be null, returns NeoVIMICErrTypeInvalidParameter if it is. Set to how many devices are found.
///
/// @return           NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
#[no_mangle]
extern "C" fn mic2_find(devices: *const NeoVIMIC, length: *mut u32, api_version: u32, neovi_mic_size: u32) -> NeoVIMICErrType {
    if devices.is_null() || length.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }
    // Check if the version is compatible
    if api_version != MIC2_API_VERSION {
        return NeoVIMICErrType::NeoVIMICErrTypeVersionMismatch;
    }
    // make sure we have enough space
    if neovi_mic_size < std::mem::size_of::<NeoVIMIC>() as u32 {
        return NeoVIMICErrType::NeoVIMICErrTypeSizeMismatch;
    }
    // Find all the attached neovi MIC2s
    let mut found_devices = match mic::find_neovi_mics() {
        Ok(d) => d
            .into_iter()
            .map(|x| NeoVIMICHandle::from(x))
            .collect::<Vec<NeoVIMICHandle>>(),
        Err(_e) => return NeoVIMICErrType::NeoVIMICErrTypeFailure,
    };
    // Set the length of the devices array
    let length = unsafe { &mut *length };
    *length = std::cmp::min(*length, found_devices.len() as u32);
    // Convert the devices array to a mutable slice
    let devices = unsafe { slice::from_raw_parts_mut(devices as *mut NeoVIMIC, *length as usize) };
    //for i in 0..devices.len() {
    for (i, device) in devices.iter_mut().enumerate() {
        device.version = api_version;
        device.size = neovi_mic_size;
        // Copy the serial number over
        device.serial_number.fill(0);
        let sn = CString::new(found_devices[i].inner.lock().unwrap().get_serial_number()).unwrap();
        let sn_len = std::cmp::min(sn.as_bytes_with_nul().len(), device.serial_number.len() - 1);
        let serial_number_slice = device.serial_number.as_mut_slice();
        unsafe {
            serial_number_slice[..sn_len]
                .copy_from_slice(slice::from_raw_parts(sn.as_ptr(), sn_len));
        }
        // Copy the handle over
        let found_device = found_devices.swap_remove(i);
        device.handle = Box::into_raw(Box::new(found_device)) as *mut _;
    }

    NeoVIMICErrType::NeoVIMICErrTypeSuccess
}

/// Check if the neoVI MIC2 has GPS functionality.
///
/// @param device    Pointer to aNeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
/// @param has_gps   Pointer to a bool. Set to true if has GPS, false if not. Returns NeoVIMICErrTypeInvalidParameter if nullptr
///
/// @return          NeoVIMICErrTypeSuccess if successful.
#[no_mangle]
extern "C" fn mic2_has_gps(device: *const NeoVIMIC, has_gps: *mut bool) -> NeoVIMICErrType {
    if device.is_null() || has_gps.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }
    unsafe { *has_gps = false };

    let neovi_mic = unsafe { 
        let device = &*device;
        let handle = &*(device.handle as *mut NeoVIMICHandle);
        handle.inner.lock().unwrap()
    };
    unsafe { *has_gps = neovi_mic.has_gps() };
    NeoVIMICErrType::NeoVIMICErrTypeSuccess
}

/// Open the IO interface on the device.
///
/// @param device    Pointer to a NeoVIMIC struct. Returns NeoVIMICErrTypeInvalidParameter if nullptr
///
/// @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
#[no_mangle]
extern "C" fn mic2_io_open(device: *const NeoVIMIC) -> NeoVIMICErrType {
    if device.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }
    let neovi_mic = unsafe { 
        let device = &*device;
        let handle = &*(device.handle as *mut NeoVIMICHandle);
        handle.inner.lock().unwrap()
    };
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
extern "C" fn mic2_io_close(device: *const NeoVIMIC) -> NeoVIMICErrType {
    if device.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }
    let neovi_mic = unsafe { 
        let device = &*device;
        let handle = &*(device.handle as *mut NeoVIMICHandle);
        handle.inner.lock().unwrap()
    };
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
extern "C" fn mic2_io_is_open(device: *const NeoVIMIC, is_open: *mut bool) -> NeoVIMICErrType {
    if device.is_null() || is_open.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }
    unsafe { *is_open = false };

    let neovi_mic = unsafe { 
        let device = &*device;
        let handle = &*(device.handle as *mut NeoVIMICHandle);
        handle.inner.lock().unwrap()
    };
    match neovi_mic.io_is_open() {
        Ok(b) => {
            unsafe { *is_open = b };
            NeoVIMICErrType::NeoVIMICErrTypeSuccess
        }
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
extern "C" fn mic2_io_buzzer_enable(device: *const NeoVIMIC, enable: bool) -> NeoVIMICErrType {
    if device.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }
    let neovi_mic = unsafe { 
        let device = &*device;
        let handle = &*(device.handle as *mut NeoVIMICHandle);
        handle.inner.lock().unwrap()
    };
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
extern "C" fn mic2_io_buzzer_is_enabled(
    device: *const NeoVIMIC,
    is_enabled: *mut bool,
) -> NeoVIMICErrType {
    if device.is_null() || is_enabled.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }
    unsafe { *is_enabled = false };

    let neovi_mic = unsafe { 
        let device = &*device;
        let handle = &*(device.handle as *mut NeoVIMICHandle);
        handle.inner.lock().unwrap()
    };
    match neovi_mic.io_buzzer_is_enabled() {
        Ok(b) => {
            unsafe { *is_enabled = b };
            NeoVIMICErrType::NeoVIMICErrTypeSuccess
        }
        Err(_e) => NeoVIMICErrType::NeoVIMICErrTypeFailure,
    }
}

/// Enable the IO GPS LED on the device.
///
/// @param device    Pointer to aNeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
/// @param enable   Set to true to enable, false if not.
///
/// @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
#[no_mangle]
extern "C" fn mic2_io_gpsled_enable(device: *const NeoVIMIC, enable: bool) -> NeoVIMICErrType {
    if device.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }
    let neovi_mic = unsafe { 
        let device = &*device;
        let handle = &*(device.handle as *mut NeoVIMICHandle);
        handle.inner.lock().unwrap()
    };
    match neovi_mic.io_gpsled_enable(enable) {
        Ok(_) => NeoVIMICErrType::NeoVIMICErrTypeSuccess,
        Err(_e) => NeoVIMICErrType::NeoVIMICErrTypeFailure,
    }
}

/// Check if the IO GPS LED on the device is enabled.
///
/// @param device    Pointer to aNeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
/// @param is_enabled   Pointer to a bool. Set to true if enabled, false if not. Returns NeoVIMICErrTypeInvalidParameter if nullptr
///
/// @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
#[no_mangle]
extern "C" fn mic2_io_gpsled_is_enabled(
    device: *const NeoVIMIC,
    is_enabled: *mut bool,
) -> NeoVIMICErrType {
    if device.is_null() || is_enabled.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }
    unsafe { *is_enabled = false };

    let neovi_mic = unsafe { 
        let device = &*device;
        let handle = &*(device.handle as *mut NeoVIMICHandle);
        handle.inner.lock().unwrap()
    };
    match neovi_mic.io_gpsled_is_enabled() {
        Ok(b) => {
            unsafe { *is_enabled = b };
            NeoVIMICErrType::NeoVIMICErrTypeSuccess
        }
        Err(_e) => NeoVIMICErrType::NeoVIMICErrTypeFailure,
    }
}

/// Check if the IO Button on the device is enabled.
///
/// @param device    Pointer to aNeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
/// @param is_pressed   Pointer to a bool. Set to true if enabled, false if not. Returns NeoVIMICErrTypeInvalidParameter if nullptr
///
/// @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
#[no_mangle]
extern "C" fn mic2_io_button_is_pressed(
    device: *const NeoVIMIC,
    is_pressed: *mut bool,
) -> NeoVIMICErrType {
    if device.is_null() || is_pressed.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }
    unsafe { *is_pressed = false };

    let neovi_mic = unsafe { 
        let device = &*device;
        let handle = &*(device.handle as *mut NeoVIMICHandle);
        handle.inner.lock().unwrap()
    };
    match neovi_mic.io_button_is_pressed() {
        Ok(b) => {
            unsafe { *is_pressed = b };
            NeoVIMICErrType::NeoVIMICErrTypeSuccess
        }
        Err(_e) => NeoVIMICErrType::NeoVIMICErrTypeFailure,
    }
}

/// Free the NeoVIMIC object. This must be called when finished otherwise a memory leak will occur.
///
/// @param device    Pointer to aNeoVIMIC structs. Okay to pass a nullptr.
///
/// @return          None
#[no_mangle]
unsafe extern "C" fn mic2_free(device: *const NeoVIMIC) -> () {
    if device.is_null() {
        return;
    }
    unsafe { 
        let device = &*device;
        std::mem::drop(Box::from_raw(device.handle))
    };
}
