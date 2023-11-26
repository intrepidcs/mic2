use std::borrow::BorrowMut;

use crate::{types::Result, io::{IO, IOBitMode}};
use rusb::{self, GlobalContext};

/// Intrepid Control Systems, Inc. USB Vendor ID.
const NEOVI_MIC_VID: u16 = 0x93c;
/// neoVI MIC2 Product ID, shared with ValueCAN3 PID.
const NEOVI_MIC_PID: u16 = 0x601;

#[derive(Debug, Clone, PartialEq)]
pub enum UsbDeviceType {
    MicrochipHub,
    FT245R,
    FT245RAccessDenied,
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
        match (vid, pid) {
            (0x0424, 0x2514) => UsbDeviceType::MicrochipHub,
            (0x93C, 0x601) => UsbDeviceType::FT245R,
            (0x8BB, 0x2912) => UsbDeviceType::Audio,
            (0x1546, 0x1A8) => UsbDeviceType::GPS,
            _ => UsbDeviceType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NeoVIMIC {
    usb_hub: UsbDeviceInfo,
    usb_children: Vec<UsbDeviceInfo>,
    index: u32,
    pub io: Option<IO>,
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
        let mut io = None;
        let mut usb_children = Vec::new();
        for device in rusb::devices().unwrap().iter() {
            let parent = device.get_parent();
            if parent.is_none() {
                continue;
            }
            let parent = UsbDeviceInfo::from_rusb_device(&parent.unwrap());
            if parent == *usb_hub {
                let mut child: UsbDeviceInfo = UsbDeviceInfo::from_rusb_device(&device);
                // Lets attempt to open the device and get the serial number
                if child.device_type == UsbDeviceType::FT245R {
                    let serial_number = match &device.open() {
                        Ok(handle) => handle
                            .read_serial_number_string_ascii(&device.device_descriptor().unwrap())
                            .unwrap(),
                        Err(e) => {
                            // Probably an access denied error, udev rules correct?
                            format!("{e}").into()
                        },
                    };
                    child = UsbDeviceInfo {
                        serial_number: Some(serial_number.into()),
                        ..child
                    };
                    io = IO::from(&child).ok();
                }
                usb_children.push(child);
            }
        }
        // Create the IO device

        devices.push(NeoVIMIC {
            usb_hub: usb_hub.clone(),
            usb_children,
            index: i as u32,
            io,
        });
    }
    Ok(devices)
}

impl NeoVIMIC {
    /// Returns true if this neoVI MIC2 has GPS capabilities, false otherwise
    pub fn has_gps(&self) -> bool {
        for child in &self.usb_children {
            if child.device_type == UsbDeviceType::GPS {
                return true;
            }
        }
        false
    }

    pub fn get_serial_number(&self) -> String {
        for child in &self.usb_children {
            if child.device_type == UsbDeviceType::FT245R {
                return child.serial_number.clone().unwrap_or_default();
            }
        }
        "".into()
    }

    pub fn io_open(&self) -> Result<()> {
        self.io.as_ref().expect("IO device not available").open()
    }

    pub fn io_close(&self) -> Result<()> {
        self.io.as_ref().expect("IO device not available").borrow_mut().close()
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
        self.io.as_ref().expect("IO device not available").set_bitmode(bit_mode)
    }

    pub fn io_buzzer_is_enabled(&self) -> Result<bool> {
        let pins = self.io.as_ref().expect("IO device not available").read_pins()?;
        Ok(pins & IOBitMode::Buzzer == IOBitMode::Buzzer)
    }

    pub fn io_gpsled_enable(&self, enabled: bool) -> Result<()> {
        let bit_mode = if enabled {
            IOBitMode::GPSLedMask | IOBitMode::GPSLed
        } else {
            IOBitMode::GPSLedMask.into()
        };
        self.io.as_ref().expect("IO device not available").set_bitmode(bit_mode)
    }

    pub fn io_gpsled_is_enabled(&self) -> Result<bool> {
        let pins = self.io.as_ref().expect("IO device not available").read_pins()?;
        Ok(pins & IOBitMode::GPSLed == IOBitMode::GPSLed)
    }

    pub fn io_button_is_pressed(&self) -> Result<bool> {
        let pins = self.io.as_ref().expect("IO device not available").read_pins()?;
        Ok(pins & IOBitMode::Button == IOBitMode::Button)
    }

    fn get_first_child(&self, device_type: UsbDeviceType) -> Result<&UsbDeviceInfo> {
        for usb_child in &self.usb_children {
            if usb_child.device_type == device_type {
                return Ok(usb_child);
            }
        }
        Err(crate::types::Error::InvalidDevice("No valid device type found".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_neovi_mics() {
        let devices = find_neovi_mics().expect("Expected at least one neoVI MIC2!");
        println!("{devices:#X?}");

        println!("Found {} device(s)", devices.len());
        for device in &devices {
            print!("neoVI MIC2 {} ", {device.get_serial_number()});
            match device.has_gps() {
                true => println!("with GPS"),
                false => println!(""),
            }
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
            assert_eq!(device.io_buzzer_is_enabled().unwrap(), true, "Buzzer should be enabled and its not!");
            std::thread::sleep(std::time::Duration::from_secs_f64(0.1f64));
            device.io_buzzer_enable(false).unwrap();
            assert_eq!(device.io_buzzer_is_enabled().unwrap(), false, "Buzzer should be disabled and its not!");

            // Test the GPS LED
            device.io_gpsled_enable(true).unwrap();
            assert_eq!(device.io_gpsled_is_enabled().unwrap(), true, "GPS LED should be enabled and its not!");
            std::thread::sleep(std::time::Duration::from_secs_f64(0.1f64));
            device.io_gpsled_enable(false).unwrap();
            assert_eq!(device.io_gpsled_is_enabled().unwrap(), false, "GPS LED should be disabled and its not!");

            // Test the button
            assert_eq!(device.io_button_is_pressed().unwrap(), false, "Button shouldn't be pressed and it is!");

            device.io_close().unwrap();
        }
    }
}
