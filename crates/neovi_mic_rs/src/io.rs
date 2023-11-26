use std::cell::RefCell;

use crate::{mic::UsbDeviceInfo, types::Result};
use enumflags2::{bitflags, BitFlags};
use libftdi1_sys::{
    ftdi_context, ftdi_new, ftdi_read_pins, ftdi_set_bitmode, ftdi_usb_close,
    ftdi_usb_open_bus_addr,
};

/// FTDI CBUS Pin Configuration
///
/// CBUS0 = Buzzer (Output - High = on)
/// CBUS1 = Button (Input)
/// CBUS2 = GPS LED (Output - High = on)
/// CBUS3 = N/C
#[bitflags(default = BuzzerMask | ButtonMask | GPSLedMask)]
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum IOBitMode {
    Buzzer = 0x1,
    Button = 0x2,
    GPSLed = 0x4,
    CBUS3 = 0x8,
    BuzzerMask = 0x10,
    ButtonMask = 0x20,
    GPSLedMask = 0x40,
    CBUS3Mask = 0x80,
}

impl IOBitMode {
    pub fn from_bits(value: u8) -> Result<BitFlags::<Self>> {
        // TODO: This should handle unwrap better
        Ok(BitFlags::<Self>::from_bits(value).unwrap())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IO {
    usb_device_info: UsbDeviceInfo,
    context: RefCell<*mut ftdi_context>,
    is_open: RefCell<bool>,
}

impl Default for IO {
    fn default() -> Self {
        Self {
            usb_device_info: UsbDeviceInfo::default(),
            context: RefCell::new(std::ptr::null_mut()),
            is_open: RefCell::new(false),
        }
    }
}

impl Drop for IO {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

impl IO {
    pub fn from(usb_device_info: UsbDeviceInfo) -> Result<Self> {
        let context = unsafe { ftdi_new() };
        if context.is_null() {
            return Err(crate::types::Error::CriticalError(
                "Failed to initialize new ftdi context".into(),
            ));
        }
        Ok(Self {
            usb_device_info,
            context: RefCell::new(context),
            is_open: RefCell::new(false),
        })
    }

    pub fn is_open(&self) -> bool {
        *self.is_open.borrow()
    }

    pub fn open(&self) -> Result<()> {
        let result = unsafe {
            ftdi_usb_open_bus_addr(
                *self.context.borrow_mut(),
                self.usb_device_info.bus_number,
                self.usb_device_info.address,
            )
        };
        let error_code: String = match result {
            0 => "all fine".into(),
            -1 => "usb_find_busses() failed".into(),
            -2 => "usb_find_devices() failed".into(),
            -3 => "usb device not found".into(),
            -4 => "unable to open device".into(),
            -5 => "unable to claim device".into(),
            -6 => "reset failed".into(),
            -7 => "set baudrate failed".into(),
            -8 => "get product description failed".into(),
            -9 => "get serial number failed".into(),
            -10 => "unable to close device".into(),
            -11 => "ftdi context invalid".into(),
            -12 => "libusb_get_device_list() failed".into(),
            _ => format!("Unknown error code: {result}").into(),
        };
        if result != 0 {
            return Err(crate::types::Error::CriticalError(error_code));
        };
        *self.is_open.borrow_mut() = true;
        Ok(())
    }

    pub fn close(&self) -> Result<()> {
        let result = unsafe { ftdi_usb_close(*self.context.borrow_mut()) };
        let error_code: String = match result {
            0 => "all fine".into(),
            -1 => "usb_release failed".into(),
            -3 => "ftdi context invalid".into(),
            _ => format!("Unknown error code: {result}").into(),
        };
        if result != 0 {
            return Err(crate::types::Error::CriticalError(error_code));
        };
        *self.is_open.borrow_mut() = false;
        Ok(())
    }

    /// Enable/disable bitbang modes.
    /// bitmask	Bitmask to configure lines. HIGH/ON value configures a line as output.
    /// mode	Bitbang mode: use the values defined in ftdi_mpsse_mode
    ///
    /// CBUS0 = Buzzer
    /// CBUS1 = Button
    /// CBUS2 = GPS LED
    /// CBUS3 = N/C
    ///
    pub fn set_bitmode_raw(&self, bitmask: u8) -> Result<()> {
        let result = unsafe {
            ftdi_set_bitmode(
                *self.context.borrow_mut(),
                bitmask,
                libftdi1_sys::ftdi_mpsse_mode::BITMODE_CBUS
                    .0
                    .try_into()
                    .unwrap(),
            )
        };
        let error_code: String = match result {
            0 => "all fine".into(),
            -1 => "can't enable bitbang mode".into(),
            -2 => "USB device unavailable".into(),
            _ => format!("Unknown error code: {result}").into(),
        };
        if result != 0 {
            return Err(crate::types::Error::CriticalError(error_code));
        };
        Ok(())
    }

    /// Enable/disable bitbang modes.
    /// bitmask	Bitmask to configure lines. HIGH/ON value configures a line as output.
    /// mode	Bitbang mode: use the values defined in ftdi_mpsse_mode
    ///
    /// CBUS0 = Buzzer
    /// CBUS1 = Button
    /// CBUS2 = GPS LED
    /// CBUS3 = N/C
    ///
    pub fn set_bitmode(&self, bitmask: BitFlags<IOBitMode>) -> Result<()> {
        self.set_bitmode_raw(bitmask.bits())
    }

    /// Directly read pin state, circumventing the read buffer. Useful for bitbang mode.
    ///
    /// CBUS0 = Buzzer
    /// CBUS1 = Button
    /// CBUS2 = GPS LED
    /// CBUS3 = N/C
    ///
    pub fn read_pins_raw(&self) -> Result<u8> {
        let mut pins: u8 = 0;
        let result = unsafe { ftdi_read_pins(*self.context.borrow_mut(), &mut pins) };
        let error_code: String = match result {
            0 => "all fine".into(),
            -1 => "read pins failed".into(),
            -2 => "USB device unavailable".into(),
            _ => format!("Unknown error code: {result}").into(),
        };
        if result != 0 {
            return Err(crate::types::Error::CriticalError(error_code));
        };
        // the bitbang_cbus.c example has all the mask values masked, I'm guessing it doesn't
        // read back correctly.
        pins &= 0xf;
        Ok(pins)
    }

    pub fn read_pins(&self) -> Result<BitFlags<IOBitMode>> {
        Ok(
            BitFlags::<IOBitMode>::from_bits(self.read_pins_raw()?).unwrap()
        )
    }

    pub fn get_usb_device_info(&self) -> &UsbDeviceInfo {
        &self.usb_device_info
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use crate::mic::find_neovi_mics;

    use super::*;

    // Since we are dealing with Handles we can only open the device one at a time.
    // cargo runs all tests in parallel.
    static LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_io_ref() -> Result<()> {
        let _lock = LOCK.lock().unwrap();

        let mut devices = find_neovi_mics()?;
        if devices.len() == 0 {
            panic!("Need at least one neoVI MIC connected, found 0 devices...");
        }
        for device in &mut devices {
            let io = device.io.as_ref().expect("IO is not valid");

            // Open and check
            io.open()?;
            assert_eq!(io.is_open(), true);

            // Test the buzzer
            io.set_bitmode(IOBitMode::BuzzerMask | IOBitMode::Buzzer)?;
            std::thread::sleep(std::time::Duration::from_secs_f64(0.1f64));
            let pins = io.read_pins()?;
            assert_eq!(pins, IOBitMode::Buzzer, "Expected Buzzer to be enabled!");

            // Test the GPS LED
            io.set_bitmode(IOBitMode::GPSLedMask | IOBitMode::GPSLed)?;
            std::thread::sleep(std::time::Duration::from_secs_f64(0.1f64));
            let pins = io.read_pins()?;
            assert_eq!(pins, IOBitMode::GPSLed, "Expected GPS LED to be enabled!");

            // Turn everything off
            io.set_bitmode(IOBitMode::GPSLedMask | IOBitMode::ButtonMask | IOBitMode::BuzzerMask)?;
            std::thread::sleep(std::time::Duration::from_secs_f64(0.1f64));
            let pins = io.read_pins()?;
            assert_eq!(pins.bits(), 0u8, "Expected GPS LED to be enabled!");

            // Close and check
            io.close()?;
            assert_eq!(io.is_open(), false);
        }
        Ok(())
    }
    
    #[test]
    fn test_io_owned() -> Result<()> {
        let _lock = LOCK.lock().unwrap();

        let mut devices = find_neovi_mics()?;
        if devices.len() == 0 {
            panic!("Need at least one neoVI MIC connected, found 0 devices...");
        }
        for device in &mut devices {
            let io = device.io.to_owned().expect("IO is not valid");

            // Open and check
            io.open()?;
            assert_eq!(io.is_open(), true);

            // Test the buzzer
            io.set_bitmode(IOBitMode::BuzzerMask | IOBitMode::Buzzer)?;
            std::thread::sleep(std::time::Duration::from_secs_f64(0.1f64));
            let pins = io.read_pins()?;
            assert_eq!(pins, IOBitMode::Buzzer, "Expected Buzzer to be enabled!");

            // Test the GPS LED
            io.set_bitmode(IOBitMode::GPSLedMask | IOBitMode::GPSLed)?;
            std::thread::sleep(std::time::Duration::from_secs_f64(0.1f64));
            let pins = io.read_pins()?;
            assert_eq!(pins, IOBitMode::GPSLed, "Expected GPS LED to be enabled!");

            // Turn everything off
            io.set_bitmode(IOBitMode::GPSLedMask | IOBitMode::ButtonMask | IOBitMode::BuzzerMask)?;
            std::thread::sleep(std::time::Duration::from_secs_f64(0.1f64));
            let pins = io.read_pins()?;
            assert_eq!(pins.bits(), 0u8, "Expected GPS LED to be enabled!");

            // Close and check
            io.close()?;
            assert_eq!(io.is_open(), false);
        }
        Ok(())
    }
}
