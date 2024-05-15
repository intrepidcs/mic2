#[cfg(feature = "io")]
use crate::io::{IOBitMode, IO};
use crate::{
    audio::Audio,
    gps::GPSDevice,
    nmea::types::GPSInfo,
    types::{Error, Result},
};
use rusb::{self, GlobalContext};

/// Intrepid Control Systems, Inc. USB Vendor ID.
const NEOVI_MIC_VID: u16 = 0x93c;
/// neoVI MIC2 Product ID, shared with ValueCAN3 PID.
const NEOVI_MIC_PID: u16 = 0x601;
/// neoVI MIC2 USB Hub Vedor ID.
const NEOVI_MIC_HUB_VID: u16 = 0x424;
/// neoVI MIC2 USB Hub Product ID.
const NEOVI_MIC_HUB_PID: u16 = 0x2514;
/// neoVI MIC2 Audio Vedor ID.
const NEOVI_MIC_AUDIO_VID: u16 = 0x8BB;
/// neoVI MIC2 Audio Product ID.
const NEOVI_MIC_AUDIO_PID: u16 = 0x2912;
/// neoVI MIC2 GPS Vedor ID.
const NEOVI_MIC_GPS_VID: u16 = 0x1546;
/// neoVI MIC2 GPS Product ID.
const NEOVI_MIC_GPS_PID: u16 = 0x1A8;

#[derive(Debug, Clone, PartialEq)]
pub enum UsbDeviceType {
    MicrochipHub,
    FT245R,
    GPS,
    Audio,
    Unknown,
}

impl Default for UsbDeviceType {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UsbDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub bus_number: u8,
    pub address: u8,
    pub device_type: UsbDeviceType,
    pub serial_number: Option<String>,
}

impl UsbDeviceInfo {
    pub fn from_rusb_device(
        device: &rusb::Device<GlobalContext>,
        serial_number: Option<String>,
    ) -> Self {
        let device_desc = device.device_descriptor().unwrap();
        let vendor_id = device_desc.vendor_id();
        let product_id = device_desc.product_id();
        Self {
            vendor_id,
            product_id,
            bus_number: device.bus_number(),
            address: device.address(),
            device_type: Self::usb_device_type_from_vid_pid(&vendor_id, &product_id),
            serial_number: serial_number,
        }
    }

    fn usb_device_type_from_vid_pid(vid: &u16, pid: &u16) -> UsbDeviceType {
        match (*vid, *pid) {
            (NEOVI_MIC_HUB_VID, NEOVI_MIC_HUB_PID) => UsbDeviceType::MicrochipHub,
            (NEOVI_MIC_VID, NEOVI_MIC_PID) => UsbDeviceType::FT245R,
            (NEOVI_MIC_AUDIO_VID, NEOVI_MIC_AUDIO_PID) => UsbDeviceType::Audio,
            (NEOVI_MIC_GPS_VID, NEOVI_MIC_GPS_PID) => UsbDeviceType::GPS,
            _ => UsbDeviceType::Unknown,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct NeoVIMIC {
    /// Index of the neoVI MIC, starts at 0. 2nd device would be 1.
    pub index: u32,
    /// Information on the USB hub inside the neoVI MIC2.
    usb_hub: UsbDeviceInfo,
    /// FTDI USB info.
    io_usb_info: Option<UsbDeviceInfo>,
    /// Audio USB info.
    audio_usb_info: Option<UsbDeviceInfo>,
    /// GPS USB info.
    gps_usb_info: Option<UsbDeviceInfo>,
    /// Extra USB devices plugged into the USB hub.
    extra_usb_info: Vec<UsbDeviceInfo>,
    /// IO device attached to the USB Hub.
    #[cfg(feature = "io")]
    io: Option<IO>,
    /// Audio Device attached to the USB Hub.
    audio: Option<Audio>,
    /// GPS Device attached to the USB Hub.
    gps: Option<GPSDevice>,
}

pub fn find_neovi_mics() -> Result<Vec<NeoVIMIC>> {
    let mut usb_hubs = Vec::new();

    // Find all potential neoVI MIC2 USB hubs
    // 0424:2514 Microchip Technology, Inc. (formerly SMSC) USB 2.0 Hub
    for rusb_device in rusb::devices().unwrap().iter() {
        let device = UsbDeviceInfo::from_rusb_device(&rusb_device, None);
        // Are we the hub? 0424:2514 Microchip Technology, Inc. (formerly SMSC) USB 2.0 Hub
        if device.vendor_id == 0x0424 || device.product_id == 0x2514 {
            usb_hubs.push(device);
        }
    }

    let mut devices = Vec::new();
    // Find all children attached to all the hubs
    for (i, usb_hub) in usb_hubs.iter().enumerate() {
        // define all the UsbDeviceInfo on the hub
        let mut io_usb_info = None;
        let mut audio_usb_info = None;
        let mut gps_usb_info = None;
        let mut extra_usb_info: Vec<UsbDeviceInfo> = Vec::new();
        // define all the actual objects that reflect the UsbDeviceInfo
        #[cfg(feature = "io")]
        let mut io = None;
        let mut audio = None;
        let mut gps = None;
        // Audio devices are kind of a pain to link to the actual hub so we are just
        // going to match indexes on how they are found on the system. Index 0 neoVI MIC2 should match up
        // to Index 1 of the audio codecs found.
        let audio_devices = match Audio::find_neovi_mic2_audio() {
            Ok(devs) => devs,
            Err(e) => {
                println!("{}", e);
                Vec::new()
            }
        };
        // Find all devices attached to the hub
        for device in rusb::devices().unwrap().iter().filter(|d| {
            // Get the parent of the device, can't proceed if we don't have a parent.
            match d.get_parent() {
                Some(parent) => UsbDeviceInfo::from_rusb_device(&parent, None) == *usb_hub,
                None => false,
            }
        }) {
            // Grab the USB vendor/product ID of the device, continue if we can't get it.
            let (vendor_id, product_id) = match device.device_descriptor() {
                Ok(d) => (d.vendor_id(), d.product_id()),
                Err(_) => continue,
            };
            // match up the VID/PID to a UsbDeviceType
            match UsbDeviceInfo::usb_device_type_from_vid_pid(&vendor_id, &product_id) {
                UsbDeviceType::MicrochipHub => {}
                UsbDeviceType::FT245R => {
                    // Grab the serial number before we create the UsbDeviceInfo
                    let serial_number = match &device.open() {
                        Ok(handle) => handle
                            .read_serial_number_string_ascii(&device.device_descriptor().unwrap())
                            .unwrap(),
                        Err(e) => {
                            // Probably an access denied error, udev rules correct?
                            format!("{e}").into()
                        }
                    };
                    let usb_info = UsbDeviceInfo::from_rusb_device(&device, Some(serial_number));
                    cfg_if::cfg_if! {
                        if #[cfg(feature = "io")] {
                            io = IO::from(usb_info.clone()).ok();
                        }
                    }
                    io_usb_info = Some(usb_info);
                }
                UsbDeviceType::GPS => {
                    gps_usb_info = Some(UsbDeviceInfo::from_rusb_device(&device, None));
                    gps = GPSDevice::find(&vendor_id, &product_id);
                }
                UsbDeviceType::Audio => {
                    audio_usb_info = Some(UsbDeviceInfo::from_rusb_device(&device, None));
                    // See audio_device declaration above for information on how we are
                    // matching indexes here.
                    audio = match audio_devices.get(i) {
                        Some(audio_device) => Some(audio_device.clone()),
                        None => None,
                    };
                }
                UsbDeviceType::Unknown => {
                    extra_usb_info.push(UsbDeviceInfo::from_rusb_device(&device, None));
                }
            };
        }
        // Create the IO device
        devices.push(NeoVIMIC {
            index: i as u32,
            usb_hub: usb_hub.clone(),
            io_usb_info,
            audio_usb_info,
            gps_usb_info,
            extra_usb_info,
            #[cfg(feature="io")]
            io,
            audio,
            gps,
        });
    }
    Ok(devices)
}

impl NeoVIMIC {
    /// Returns true if this neoVI MIC2 has GPS capabilities, false otherwise
    pub fn has_gps(&self) -> bool {
        self.gps_usb_info.is_some()
    }

    pub fn get_serial_number(&self) -> String {
        match self.io_usb_info.as_ref() {
            Some(info) => info.serial_number.clone().unwrap_or_default(),
            None => "".into(),
        }
    }

    pub fn get_usb_hub_info(&self) -> &UsbDeviceInfo {
        &self.usb_hub
    }

    pub fn get_usb_io_info(&self) -> &Option<UsbDeviceInfo> {
        &self.io_usb_info
    }

    pub fn get_usb_audio_info(&self) -> &Option<UsbDeviceInfo> {
        &self.audio_usb_info
    }

    pub fn get_usb_gps_info(&self) -> &Option<UsbDeviceInfo> {
        &self.gps_usb_info
    }

    pub fn get_usb_extra_info(&self) -> &Vec<UsbDeviceInfo> {
        &self.extra_usb_info
    }

    pub fn io_open(&self) -> Result<()> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "io")] {
                match self.io.as_ref() {
                    Some(io) => io.open(),
                    None => Err(Error::NotSupported("IO device not available.".to_string())),
                }
            } else {
                Err(Error::NotSupported("io feature not enabled".to_string()))
            }
        }
    }

    pub fn io_close(&self) -> Result<()> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "io")] {
                match self.io.as_ref() {
                    Some(io) => io.close(),
                    None => Err(Error::NotSupported("IO device not available.".to_string())),
                }
            } else {
                Err(Error::NotSupported("io feature not enabled".to_string()))
            }
        }
    }

    pub fn io_is_open(&self) -> Result<bool> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "io")] {
                match self.io.as_ref() {
                    Some(io) => Ok(io.is_open()),
                    None => Err(Error::NotSupported("IO device not available.".to_string())),
                }
            } else {
                Err(Error::NotSupported("io feature not enabled".to_string()))
            }
        }
    }

    pub fn io_buzzer_enable(&self, enabled: bool) -> Result<()> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "io")] {
                let bit_mode: enumflags2::BitFlags<IOBitMode, u8> = if enabled {
                    IOBitMode::BuzzerMask | IOBitMode::Buzzer
                } else {
                    IOBitMode::BuzzerMask.into()
                };
                match self.io.as_ref() {
                    Some(io) => io.set_bitmode(bit_mode),
                    None => Err(Error::NotSupported("IO device not available.".to_string())),
                }
            } else {
                let _ = enabled;
                Err(Error::NotSupported("io feature not enabled".to_string()))
            }
        }
    }

    pub fn io_buzzer_is_enabled(&self) -> Result<bool> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "io")] {
                let pins = match self.io.as_ref() {
                    Some(io) => io.read_pins()?,
                    None => return Err(Error::NotSupported("IO device not available.".to_string())),
                };
                Ok(pins & IOBitMode::Buzzer == IOBitMode::Buzzer)
            } else {
                Err(Error::NotSupported("io feature not enabled".to_string()))
            }
        }
    }

    pub fn io_gpsled_enable(&self, enabled: bool) -> Result<()> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "io")] {
                let bit_mode = if enabled {
                    IOBitMode::GPSLedMask | IOBitMode::GPSLed
                } else {
                    IOBitMode::GPSLedMask.into()
                };
                match self.io.as_ref() {
                    Some(io) => io.set_bitmode(bit_mode),
                    None => Err(Error::NotSupported("IO device not available.".to_string())),
                }
            } else {
                let _ = enabled;
                Err(Error::NotSupported("io feature not enabled".to_string()))
            }
        }
    }

    pub fn io_gpsled_is_enabled(&self) -> Result<bool> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "io")] {
                let pins = match self.io.as_ref() {
                    Some(io) => io.read_pins()?,
                    None => return Err(Error::NotSupported("IO device not available.".to_string())),
                };
                Ok(pins & IOBitMode::GPSLed == IOBitMode::GPSLed)
            } else {
                Err(Error::NotSupported("io feature not enabled".to_string()))
            }
        }
    }

    pub fn io_button_is_pressed(&self) -> Result<bool> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "io")] {
                let pins = match self.io.as_ref() {
                    Some(io) => io.read_pins()?,
                    None => return Err(Error::NotSupported("IO device not available.".to_string())),
                };
                Ok(pins & IOBitMode::Button == IOBitMode::Button)
            } else {
                Err(Error::NotSupported("io feature not enabled".to_string()))
            }
        }
    }

    pub fn audio_start(&self, sample_rate: u32) -> Result<()> {
        match &self.audio {
            Some(audio) => audio.start(sample_rate),
            None => Err(crate::types::Error::InvalidDevice(
                "Audio device isn't available".to_string(),
            )),
        }
    }

    pub fn audio_stop(&self) -> Result<()> {
        match &self.audio {
            Some(audio) => audio.stop(),
            None => Err(crate::types::Error::InvalidDevice(
                "Audio device isn't available".to_string(),
            )),
        }
    }

    pub fn audio_save(&self, fname: impl Into<String>) -> Result<()> {
        match &self.audio {
            Some(audio) => audio.save_to_file(fname),
            None => Err(crate::types::Error::InvalidDevice(
                "Audio device isn't available".to_string(),
            )),
        }
    }

    pub fn gps_open(&self) -> Result<bool> {
        match &self.gps {
            Some(gps) => gps.open(),
            None => Err(crate::types::Error::InvalidDevice(
                "GPS device isn't available".to_string(),
            )),
        }
    }

    pub fn gps_is_open(&self) -> Result<bool> {
        match &self.gps {
            Some(gps) => Ok(gps.is_open()),
            None => Err(crate::types::Error::InvalidDevice(
                "GPS device isn't available".to_string(),
            )),
        }
    }

    pub fn gps_close(&self) -> Result<()> {
        match &self.gps {
            Some(gps) => gps.close(),
            None => Err(crate::types::Error::InvalidDevice(
                "GPS device isn't available".to_string(),
            )),
        }
    }

    pub fn gps_info(&self) -> Result<GPSInfo> {
        match &self.gps {
            Some(gps) => gps.get_info(),
            None => Err(crate::types::Error::InvalidDevice(
                "GPS device isn't available".to_string(),
            )),
        }
    }

    pub fn gps_has_lock(&self) -> Result<bool> {
        match &self.gps {
            Some(gps) => gps.has_lock(),
            None => Err(crate::types::Error::InvalidDevice(
                "GPS device isn't available".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _get_devices() -> Vec<NeoVIMIC> {
        let devices = find_neovi_mics().expect("Expected at least one neoVI MIC2!");
        //println!("{devices:#X?}");
        println!("Found {} device(s)", devices.len());
        devices
    }

    #[test]
    fn test_find_neovi_mics() {
        let devices = _get_devices();
        for device in &devices {
            print!("neoVI MIC2 {} ", { device.get_serial_number() });
            match device.has_gps() {
                true => println!("with GPS"),
                false => println!(""),
            }
        }
    }

    #[test]
    fn test_hub() {
        let devices = _get_devices();
        for device in &devices {
            let hub_info = device.get_usb_hub_info();
            println!("{:#?}", hub_info);
            assert_eq!(hub_info.vendor_id, 0x424);
            assert_eq!(hub_info.product_id, 0x2514);
            assert_eq!(hub_info.device_type, UsbDeviceType::MicrochipHub);
        }
    }

    #[test]
    fn test_audio() {
        Audio::find_neovi_mic2_audio().unwrap();
    }

    #[test]
    fn test_io() {
        let devices = find_neovi_mics().expect("Expected at least one neoVI MIC2!");
        println!("{devices:#X?}");

        println!("Found {} device(s)", devices.len());
        for device in devices {
            device.io_open().unwrap();

            // Test the buzzer
            device.io_buzzer_enable(true).unwrap();
            assert_eq!(
                device.io_buzzer_is_enabled().unwrap(),
                true,
                "Buzzer should be enabled and its not!"
            );
            std::thread::sleep(std::time::Duration::from_secs_f64(0.1f64));
            device.io_buzzer_enable(false).unwrap();
            assert_eq!(
                device.io_buzzer_is_enabled().unwrap(),
                false,
                "Buzzer should be disabled and its not!"
            );

            // Test the GPS LED
            device.io_gpsled_enable(true).unwrap();
            assert_eq!(
                device.io_gpsled_is_enabled().unwrap(),
                true,
                "GPS LED should be enabled and its not!"
            );
            std::thread::sleep(std::time::Duration::from_secs_f64(0.1f64));
            device.io_gpsled_enable(false).unwrap();
            assert_eq!(
                device.io_gpsled_is_enabled().unwrap(),
                false,
                "GPS LED should be disabled and its not!"
            );

            // Test the button
            assert_eq!(
                device.io_button_is_pressed().unwrap(),
                false,
                "Button shouldn't be pressed and it is!"
            );

            device.io_close().unwrap();
        }
    }
}
