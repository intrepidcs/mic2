use core::slice;
use mic2::{mic, nmea::types::{GPSInfo, GPSSatInfo, GpsNavigationStatus, GPSDMS}};
use std::{
    ffi::{c_void, CStr, CString},
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

#[repr(C)]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CGPSSatInfo {
    /// Satellite PRN number
    pub prn: u16,
    /// Satellite status
    ///     - Not used
    ///     U Used in solution
    ///     e Ephemeris available, but not used for navigation
    pub used: bool,
    /// Satellite azimuth, range 000..359 (degrees), only valid when azimuth_valid is true
    pub azimuth: u16,
    pub azimuth_valid: bool,
    /// Satellite elevation, range 00..90 (degrees), only valid when elevation_valid is true
    pub elevation: u16,
    pub elevation_valid: bool,
    /// Signal strength (C/N0, range 0-99), blank when not tracking, only valid when snr_valid is true
    pub snr: u8,
    pub snr_valid: bool,
    /// Satellite carrier lock time, range 00..64
    ///     0 = code lock only
    ///     64 = lock for 64 seconds or more
    pub lock_time: u8,
}

impl From<GPSSatInfo> for CGPSSatInfo {
    fn from(info: GPSSatInfo) -> Self {
        Self {
            prn: info.prn,
            used: info.used,
            azimuth: info.azimuth.unwrap_or(0),
            azimuth_valid: info.azimuth.is_some(),
            elevation: info.elevation.unwrap_or(0),
            elevation_valid: info.elevation.is_some(),
            snr: info.snr.unwrap_or(0),
            snr_valid: info.snr.is_some(),
            lock_time: info.lock_time,
        }
    }
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum CGpsNavigationStatus {
    /// "NF" No Fix
    NoFix,
    /// "DR" Dead reckoning only solution
    DeadReckoningOnly,
    /// "G2" Stand alone 2D solution
    StandAlone2D,
    /// "G3" Stand alone 3D solution
    StandAlone3D,
    /// "D2" Differential 2D solution
    Differential2D,
    /// "D3" Differential 3D solution
    Differential3D,
    /// "RK" Combined GPS + dead reckoning solution
    CombinedRKGPSDeadReckoning,
    /// "TT" Time only solution
    TimeOnly,
}

impl CGpsNavigationStatus {
    pub fn from(gps_dms: GpsNavigationStatus) -> Self {
        match gps_dms {
            GpsNavigationStatus::NoFix => CGpsNavigationStatus::NoFix,
            GpsNavigationStatus::DeadReckoningOnly => CGpsNavigationStatus::DeadReckoningOnly,
            GpsNavigationStatus::StandAlone2D => CGpsNavigationStatus::StandAlone2D,
            GpsNavigationStatus::StandAlone3D => CGpsNavigationStatus::StandAlone3D,
            GpsNavigationStatus::Differential2D => CGpsNavigationStatus::Differential2D,
            GpsNavigationStatus::Differential3D => CGpsNavigationStatus::Differential3D,
            GpsNavigationStatus::CombinedRKGPSDeadReckoning => CGpsNavigationStatus::CombinedRKGPSDeadReckoning,
            GpsNavigationStatus::TimeOnly => CGpsNavigationStatus::TimeOnly,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CGPSDMS {
    pub degrees: u16,
    pub minutes: u8,
    pub seconds: u8,
}

impl From<GPSDMS> for CGPSDMS {
    fn from(dms: GPSDMS) -> Self {
        Self {
            degrees: dms.degrees,
            minutes: dms.minutes,
            seconds: dms.seconds,
        }
    }
}

#[repr(C)]
pub struct CGPSInfo {
    // UTC Time, Current time as unix timestamp since 00:00, Jan 1 1970 UTC. Zero means invalid.
    pub current_time: i64,
    /// Latitude. See [GPSDMS] for more details. Only valid if latitude_valid is true.
    pub latitude: CGPSDMS,
    pub latitude_valid: bool,
    /// N/S Indicator, N=north or S=south
    pub latitude_direction: c_char,
    /// Longitude. See [GPSDMS] for more details. Only valid if longitude_valid is true.
    pub longitude: CGPSDMS,
    pub longitude_valid: bool,
    /// E/W Indicator, E=east or W=west
    pub longitude_direction: c_char,
    /// Altitude above user datum ellipsoid (m). -1 means invalid.
    pub altitude: f64,
    /// Navigation Status. See [GpsNavigationStatus] for more details
    pub nav_stat: CGpsNavigationStatus,
    /// Horizontal accuracy estimate. -1 means invalid.
    pub h_acc: f64,
    /// Vertical accuracy estimate. -1 means invalid.
    pub v_acc: f64,
    /// Speed over ground (km/h). -1 means invalid.
    pub sog_kmh: f64,
    /// Course over ground (degrees). -1 means invalid.
    pub cog: f64,
    /// Vertical velocity, positive = downward (m/s). -1 means invalid.
    pub vvel: f64,
    /// Age of most recent DGPS corrections, empty = none available (s). -1 means invalid.
    pub age_c: f64,
    /// HDOP, Horizontal Dilution of Precision. -1 means invalid.
    pub hdop: f64,
    /// VDOP, Vertical dilution of precision. -1 means invalid.
    pub vdop: f64,
    /// TDOP, Time dilution of precision. -1 means invalid.
    pub tdop: f64,
    /// Number of GPS/GLONASS/Beidou satellites. Only valid indexes are defined by satellites_count.
    pub satellites: [CGPSSatInfo; 16],
    /// Number of valid GPS/GLONASS/Beidou satellites populated in satellites parameter.
    pub satellites_count: u8,
    /// Receiver clock bias (ns). -1 means invalid.
    pub clock_bias: f64,
    /// Receiver clock drift (ns/s). -1 means invalid.
    pub clock_drift: f64,
    /// Timepulse Granularity, The quantization error of the Timepulse pin (ns). -1 means invalid.
    pub timepulse_granularity: f64,
}

impl From<GPSInfo> for CGPSInfo {
    fn from(gps_info: GPSInfo) -> Self {

        let (lat_dms, lat_dir, lat_valid) = match gps_info.latitude {
            Some((dms, dir)) => (dms, dir as c_char, true),
            None => (GPSDMS { degrees: 0, minutes: 0, seconds: 0 }, char::default() as c_char, false),
        };

        let (long_dms, long_dir, long_valid) = match gps_info.longitude {
            Some((dms, dir)) => (dms, dir as c_char, true),
            None => (GPSDMS { degrees: 0, minutes: 0, seconds: 0 }, char::default() as c_char, false),
        };

        let mut info = CGPSInfo {
            current_time: match gps_info.current_time {
                Some(current_time) => current_time.timestamp(),
                None => 0,
            },
            latitude: CGPSDMS::from(lat_dms),
            latitude_valid: lat_valid,
            latitude_direction: lat_dir,
            longitude: CGPSDMS::from(long_dms),
            longitude_valid: long_valid,
            longitude_direction: long_dir,
            altitude: gps_info.altitude.unwrap_or(-1.0),
            nav_stat: match gps_info.nav_stat {
                Some(nav_stat) => CGpsNavigationStatus::from(nav_stat),
                None => CGpsNavigationStatus::from(GpsNavigationStatus::NoFix),
            },
            h_acc: gps_info.h_acc.unwrap_or(-1.0),
            v_acc: gps_info.v_acc.unwrap_or(-1.0),
            sog_kmh: gps_info.sog_kmh.unwrap_or(-1.0),
            cog: gps_info.cog.unwrap_or(-1.0),
            vvel: gps_info.vvel.unwrap_or(-1.0),
            age_c: gps_info.age_c.unwrap_or(-1.0),
            hdop: gps_info.hdop.unwrap_or(-1.0),
            vdop: gps_info.vdop.unwrap_or(-1.0),
            tdop: gps_info.tdop.unwrap_or(-1.0),
            satellites: Default::default(),
            satellites_count: gps_info.satellites.len() as u8,
            clock_bias: gps_info.clock_bias.unwrap_or(-1.0),
            clock_drift: gps_info.clock_drift.unwrap_or(-1.0),
            timepulse_granularity: gps_info.timepulse_granularity.unwrap_or(-1.0),
        };
        // Copy all the satellites into the C struct
        for (i, sat) in gps_info.satellites.into_iter().enumerate() {
            info.satellites[i] = sat.into();
        }
        info
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
    for device in devices.iter_mut() {
        // remove the device
        let found_device = found_devices.pop().unwrap();
        device.version = api_version;
        device.size = neovi_mic_size;
        // Copy the serial number over
        device.serial_number.fill(0);
        let sn = CString::new(found_device.inner.lock().unwrap().get_serial_number()).unwrap();
        let sn_len = std::cmp::min(sn.as_bytes_with_nul().len(), device.serial_number.len() - 1);
        let serial_number_slice = device.serial_number.as_mut_slice();
        unsafe {
            serial_number_slice[..sn_len]
                .copy_from_slice(slice::from_raw_parts(sn.as_ptr(), sn_len));
        }
        // Copy the handle over
        device.handle = Box::into_raw(Box::new(found_device)) as *mut _;
        
    }

    NeoVIMICErrType::NeoVIMICErrTypeSuccess
}

/// Check if the neoVI MIC2 has GPS functionality.
///
/// @param device    Pointer to a NeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
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
/// @param device    Pointer to a NeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
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
/// @param device    Pointer to a NeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
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
/// @param device    Pointer to a NeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
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
/// @param device    Pointer to a NeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
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
/// @param device    Pointer to a NeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
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
/// @param device    Pointer to a NeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
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
/// @param device    Pointer to a NeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
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

/// Starts recording audio on the device asynchronously. Call mic2_audio_stop() to stop recording.
///
/// @param device    Pointer to a NeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
/// @param sample_rate   Sample rate in Hz, typically 44100 or 48000
///
/// @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
#[no_mangle]
unsafe extern "C" fn mic2_audio_start(device: *const NeoVIMIC, sample_rate: u32) -> NeoVIMICErrType {
    if device.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }

    let neovi_mic = unsafe { 
        let device = &*device;
        let handle = &*(device.handle as *mut NeoVIMICHandle);
        handle.inner.lock().unwrap()
    };
    
    if neovi_mic.audio_start(sample_rate).is_ok() {
        NeoVIMICErrType::NeoVIMICErrTypeSuccess
    } else {
        NeoVIMICErrType::NeoVIMICErrTypeFailure
    }
}

/// Stops recording audio on the device. Call mic2_audio_start() before calling this.
///
/// @param device    Pointer to a NeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
///
/// @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
#[no_mangle]
unsafe extern "C" fn mic2_audio_stop(device: *const NeoVIMIC) -> NeoVIMICErrType {
    if device.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }

    let neovi_mic = unsafe { 
        let device = &*device;
        let handle = &*(device.handle as *mut NeoVIMICHandle);
        handle.inner.lock().unwrap()
    };
    
    if neovi_mic.audio_stop().is_ok() {
        NeoVIMICErrType::NeoVIMICErrTypeSuccess
    } else {
        NeoVIMICErrType::NeoVIMICErrTypeFailure
    }
}

/// Saves recording from the device asynchronously. Typically called after mic2_audio_stop() to save to disk.
///
/// @param device    Pointer to a NeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
/// @param path      filepath to save to. File extension determines format. (ie. .wav, .mp3, .ogg, etc)
///
/// @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
#[no_mangle]
unsafe extern "C" fn mic2_audio_save(device: *const NeoVIMIC, path: *const c_char) -> NeoVIMICErrType {
    if device.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }

    let path = CStr::from_ptr(path).to_str().unwrap();

    let neovi_mic = unsafe { 
        let device = &*device;
        let handle = &*(device.handle as *mut NeoVIMICHandle);
        handle.inner.lock().unwrap()
    };
    
    if neovi_mic.audio_save(path).is_ok() {
        NeoVIMICErrType::NeoVIMICErrTypeSuccess
    } else {
        NeoVIMICErrType::NeoVIMICErrTypeFailure
    }
}


/// Open the GPS interface on the device.
///
/// @param device    Pointer to a NeoVIMIC struct. Returns NeoVIMICErrTypeInvalidParameter if nullptr
///
/// @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
#[no_mangle]
extern "C" fn mic2_gps_open(device: *const NeoVIMIC) -> NeoVIMICErrType {
    if device.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }
    let neovi_mic = unsafe { 
        let device = &*device;
        let handle = &*(device.handle as *mut NeoVIMICHandle);
        handle.inner.lock().unwrap()
    };
    match neovi_mic.gps_open() {
        Ok(_) => NeoVIMICErrType::NeoVIMICErrTypeSuccess,
        Err(_e) => NeoVIMICErrType::NeoVIMICErrTypeFailure,
    }
}

/// Close the GPS interface on the device.
///
/// @param device    Pointer to a NeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
///
/// @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
#[no_mangle]
extern "C" fn mic2_gps_close(device: *const NeoVIMIC) -> NeoVIMICErrType {
    if device.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }
    let neovi_mic = unsafe { 
        let device = &*device;
        let handle = &*(device.handle as *mut NeoVIMICHandle);
        handle.inner.lock().unwrap()
    };
    match neovi_mic.gps_close() {
        Ok(_) => NeoVIMICErrType::NeoVIMICErrTypeSuccess,
        Err(_e) => NeoVIMICErrType::NeoVIMICErrTypeFailure,
    }
}

/// Check if the GPS interface on the device is open.
///
/// @param device    Pointer to a NeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
/// @param is_open   Pointer to a bool. Set to true if open, false if not. Returns NeoVIMICErrTypeInvalidParameter if nullptr
///
/// @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
#[no_mangle]
extern "C" fn mic2_gps_is_open(device: *const NeoVIMIC, is_open: *mut bool) -> NeoVIMICErrType {
    if device.is_null() || is_open.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }
    unsafe { *is_open = false };

    let neovi_mic = unsafe { 
        let device = &*device;
        let handle = &*(device.handle as *mut NeoVIMICHandle);
        handle.inner.lock().unwrap()
    };
    match neovi_mic.gps_is_open() {
        Ok(b) => {
            unsafe { *is_open = b };
            NeoVIMICErrType::NeoVIMICErrTypeSuccess
        }
        Err(_e) => NeoVIMICErrType::NeoVIMICErrTypeFailure,
    }
}

/// Check if the GPS interface has a lock.
///
/// @param device    Pointer to a NeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
/// @param has_lock   Pointer to a bool. Set to true if has lock, false if not. Returns NeoVIMICErrTypeInvalidParameter if nullptr
///
/// @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
#[no_mangle]
extern "C" fn mic2_gps_has_lock(device: *const NeoVIMIC, has_lock: *mut bool) -> NeoVIMICErrType {
    if device.is_null() || has_lock.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }
    unsafe { *has_lock = false };

    let neovi_mic = unsafe { 
        let device = &*device;
        let handle = &*(device.handle as *mut NeoVIMICHandle);
        handle.inner.lock().unwrap()
    };
    match neovi_mic.gps_has_lock() {
        Ok(b) => {
            unsafe { *has_lock = b };
            NeoVIMICErrType::NeoVIMICErrTypeSuccess
        }
        Err(_e) => NeoVIMICErrType::NeoVIMICErrTypeFailure,
    }
}

/// Retrieve the current GPS info.
///
/// @param device    Pointer to a NeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
/// @param info      Pointer to a CGPSInfo struct. Returns NeoVIMICErrTypeInvalidParameter if nullptr
/// @param info_size Size of the CGPSInfo struct. Returns NeoVIMICErrTypeSizeMismatch if size is smaller than expected.
/// 
/// @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
#[no_mangle]
extern "C" fn mic2_gps_info(device: *const NeoVIMIC, info: *mut CGPSInfo, info_size: usize) -> NeoVIMICErrType {
    if device.is_null() || info.is_null() {
        return NeoVIMICErrType::NeoVIMICErrTypeInvalidParameter;
    }
    if info_size < std::mem::size_of::<CGPSInfo>() {
        return NeoVIMICErrType::NeoVIMICErrTypeSizeMismatch;
    }
    let neovi_mic = unsafe { 
        let device = &*device;
        let handle = &*(device.handle as *mut NeoVIMICHandle);
        handle.inner.lock().unwrap()
    };
    match neovi_mic.gps_info() {
        Ok(gps_info) => {
            unsafe { *info = gps_info.into() };
            NeoVIMICErrType::NeoVIMICErrTypeSuccess
        }
        Err(_e) => NeoVIMICErrType::NeoVIMICErrTypeFailure,
    }
}

/// Free the NeoVIMIC object. This must be called when finished otherwise a memory leak will occur.
///
/// @param device    Pointer to a NeoVIMIC structs. Okay to pass a nullptr.
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
