use std::borrow::BorrowMut;

use crate::{
    audio::Audio, gps::GPSDevice, io::{IOBitMode, IO}, nmea::types::GPSInfo, types::Result
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
    pub fn from_rusb_device(device: &rusb::Device<GlobalContext>) -> Self {
        let device_desc = device.device_descriptor().unwrap();
        let vendor_id = device_desc.vendor_id();
        let product_id = device_desc.product_id();
        Self {
            vendor_id,
            product_id,
            bus_number: device.bus_number(),
            address: device.address(),
            device_type: Self::usb_device_type_from_vid_pid(&vendor_id, &product_id),
            serial_number: None,
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

#[derive(Debug, Default)]
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
        let device = UsbDeviceInfo::from_rusb_device(&rusb_device);
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
        let mut io = None;
        let mut audio = None;
        let mut gps = None;
        // Audio devices are kind of a pain to link to the actual hub so we are just
        // going to match indexes on how they are found on the system. Index 0 neoVI MIC2 should match up
        // to Index 1 of the audio codecs found.
        let audio_devices = match Audio::find_neovi_mic2_audio() {
            Ok(devs) => devs,
            Err(_e) => Vec::new(),
        };
        for device in rusb::devices().unwrap().iter() {
            let parent = device.get_parent();
            if parent.is_none() {
                continue;
            }
            let parent = UsbDeviceInfo::from_rusb_device(&parent.unwrap());
            if parent == *usb_hub {
                let mut child: UsbDeviceInfo = UsbDeviceInfo::from_rusb_device(&device);
                // Match up the UsbDeviceInfo to the proper member
                match &child.device_type {
                    UsbDeviceType::MicrochipHub => {}
                    UsbDeviceType::FT245R => {
                        io_usb_info = Some(child.clone());
                        io = IO::from(child.clone()).ok();
                        // Lets attempt to open the device and get the serial number
                        let serial_number = match &device.open() {
                            Ok(handle) => handle
                                .read_serial_number_string_ascii(&device.device_descriptor().unwrap())
                                .unwrap(),
                            Err(e) => {
                                // Probably an access denied error, udev rules correct?
                                format!("{e}").into()
                            }
                        };
                        // Recreate the child with serial number.
                        child = UsbDeviceInfo {
                            serial_number: Some(serial_number.into()),
                            ..child.clone()
                        };
                    }
                    UsbDeviceType::GPS => {
                        gps_usb_info = Some(child.clone());
                        gps = GPSDevice::find(&child.vendor_id, &child.product_id);
                    }
                    UsbDeviceType::Audio => {
                        audio_usb_info = Some(child.clone());
                        // See audio_device declaration above for information on how we are
                        // matching indexes here.
                        if i < audio_devices.len() {
                            audio = Some(audio_devices[i].clone());
                        }
                    }
                    UsbDeviceType::Unknown => extra_usb_info.push(child.clone()),
                }
            }
        }
        // Create the IO device

        devices.push(NeoVIMIC {
            index: i as u32,
            usb_hub: usb_hub.clone(),
            io_usb_info,
            audio_usb_info,
            gps_usb_info,
            extra_usb_info,
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
            None => "".into()
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
        self.io.as_ref().expect("IO device not available").open()
    }

    pub fn io_close(&self) -> Result<()> {
        self.io
            .as_ref()
            .expect("IO device not available")
            .borrow_mut()
            .close()
    }

    pub fn io_is_open(&self) -> Result<bool> {
        Ok(self.io.as_ref().expect("IO device not available").is_open())
    }

    pub fn io_buzzer_enable(&self, enabled: bool) -> Result<()> {
        let bit_mode = if enabled {
            IOBitMode::BuzzerMask | IOBitMode::Buzzer
        } else {
            IOBitMode::BuzzerMask.into()
        };
        self.io
            .as_ref()
            .expect("IO device not available")
            .set_bitmode(bit_mode)
    }

    pub fn io_buzzer_is_enabled(&self) -> Result<bool> {
        let pins = self
            .io
            .as_ref()
            .expect("IO device not available")
            .read_pins()?;
        Ok(pins & IOBitMode::Buzzer == IOBitMode::Buzzer)
    }

    pub fn io_gpsled_enable(&self, enabled: bool) -> Result<()> {
        let bit_mode = if enabled {
            IOBitMode::GPSLedMask | IOBitMode::GPSLed
        } else {
            IOBitMode::GPSLedMask.into()
        };
        self.io
            .as_ref()
            .expect("IO device not available")
            .set_bitmode(bit_mode)
    }

    pub fn io_gpsled_is_enabled(&self) -> Result<bool> {
        let pins = self
            .io
            .as_ref()
            .expect("IO device not available")
            .read_pins()?;
        Ok(pins & IOBitMode::GPSLed == IOBitMode::GPSLed)
    }

    pub fn io_button_is_pressed(&self) -> Result<bool> {
        let pins = self
            .io
            .as_ref()
            .expect("IO device not available")
            .read_pins()?;
        Ok(pins & IOBitMode::Button == IOBitMode::Button)
    }

    pub fn audio_start(&self, sample_rate: u32) -> Result<()> {
        match &self.audio {
            Some(audio) => audio.start(sample_rate),
            None => panic!("Audio device isn't available"),
        }
    }

    pub fn audio_stop(&self) -> Result<()> {
        match &self.audio {
            Some(audio) => audio.stop(),
            None => panic!("Audio device isn't available"),
        }
    }

    pub fn audio_save(&self, fname: impl Into<String>) -> Result<()> {
        match &self.audio {
            Some(audio) => audio.save_to_file(fname),
            None => panic!("Audio device isn't available"),
        }
    }

    pub fn gps_open(&self) -> Result<()> {
        match &self.gps {
            Some(gps) => gps.open(),
            None => panic!("GPS device isn't available"),
        }
    }

    pub fn gps_close(&self) -> Result<()> {
        match &self.gps {
            Some(gps) => gps.close(),
            None => panic!("GPS device isn't available"),
        }
    }

    pub fn gps_info(&self) -> Result<GPSInfo> {
        match &self.gps {
            Some(gps) => Ok(gps.get_info()),
            None => panic!("GPS device isn't available"),
        }
    }

    pub fn gps_has_lock(&self) -> Result<bool> {
        match &self.gps {
            Some(gps) => Ok(gps.has_lock()),
            None => panic!("GPS device isn't available"),
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
