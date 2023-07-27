use crate::{mic::UsbDeviceInfo, types::Result};
use libftdi1_sys::{
    ftdi_context, ftdi_new, ftdi_read_pins, ftdi_set_bitmode, ftdi_usb_close,
    ftdi_usb_open_bus_addr,
};

#[derive(Debug, Clone)]
pub struct IODevice {
    usb_device_info: UsbDeviceInfo,
    context: *mut ftdi_context,
    is_open: bool,
}

impl IODevice {
    pub fn from(usb_device_info: &UsbDeviceInfo) -> Result<Self> {
        let context = unsafe { ftdi_new() };
        if context.is_null() {
            return Err(crate::types::Error::CriticalError(
                "Failed to initialize new ftdi context".into(),
            ));
        }
        Ok(Self {
            usb_device_info: usb_device_info.clone(),
            context: context,
            is_open: false,
        })
    }

    pub fn open(&mut self) -> Result<()> {
        let result = unsafe {
            ftdi_usb_open_bus_addr(
                self.context,
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
        self.is_open = true;
        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        let result = unsafe { ftdi_usb_close(self.context) };
        let error_code: String = match result {
            0 => "all fine".into(),
            -1 => "usb_release failed".into(),
            -3 => "ftdi context invalid".into(),
            _ => format!("Unknown error code: {result}").into(),
        };
        if result != 0 {
            return Err(crate::types::Error::CriticalError(error_code));
        };
        self.is_open = false;
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
    pub fn set_bitmode(&self, bitmask: u8) -> Result<()> {
        let result = unsafe {
            ftdi_set_bitmode(
                self.context,
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

    /// Directly read pin state, circumventing the read buffer. Useful for bitbang mode.
    ///
    /// CBUS0 = Buzzer
    /// CBUS1 = Button
    /// CBUS2 = GPS LED
    /// CBUS3 = N/C
    ///
    pub fn read_pins(&self) -> Result<u8> {
        let mut pins: u8 = 0;
        let result = unsafe { ftdi_read_pins(self.context, &mut pins) };
        let error_code: String = match result {
            0 => "all fine".into(),
            -1 => "read pins failed".into(),
            -2 => "USB device unavailable".into(),
            _ => format!("Unknown error code: {result}").into(),
        };
        if result != 0 {
            return Err(crate::types::Error::CriticalError(error_code));
        };
        Ok(pins)
    }
}

#[cfg(test)]
mod test {
    use crate::mic::find_neovi_mics;

    use super::*;

    #[test]
    fn test_io_find_all() {
        let devices = find_neovi_mics().unwrap();
        for device in devices {
            let ftdi_device = device.get_ftdi_device().unwrap();
            let mut io_device = IODevice::from(&ftdi_device).unwrap();
            io_device.open().unwrap();
            io_device.set_bitmode(0xFF).unwrap();
            std::thread::sleep(std::time::Duration::from_secs_f64(0.35f64));
            io_device.set_bitmode(0).unwrap();
            let pins = io_device.read_pins().unwrap();
            println!("Pins: {pins:#02x}");
            io_device.close().unwrap();
        }
    }
}