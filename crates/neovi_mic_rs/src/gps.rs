use std::{str::Utf8Error, time::Duration};

use crate::types::{Error, Result};
use chrono::NaiveTime;
use serialport::{self, ErrorKind, SerialPortInfo, SerialPortType};

impl From<serialport::Error> for Error {
    fn from(value: serialport::Error) -> Self {
        match &value.kind {
            ErrorKind::NoDevice => Error::InvalidDevice("No Serial Ports found!".into()),
            _ => Error::SerialError(value),
        }
    }
}


// impl From<Utf8Error> for Error {
//     fn from(_: Utf8Error) -> Self {
//         Error::IOError(std::io::ErrorKind::InvalidData)
//     }
// }

#[derive(Debug)]
pub struct GPSDevice {
    /// Port name string similar to "/dev/ttyACM0"
    pub port_name: String,
    /// USB Vendor ID of the GPS
    pub vid: u16,
    /// USB product ID of the GPS
    pub pid: u16,
    /// baudrate of the port, typically UBLOX_DEFAULT_BAUD
    baud_rate: u32,
    /// Serial port handle with interior mutability
    port_handle: std::cell::RefCell<Option<Box<dyn serialport::SerialPort>>>,
}

impl Drop for GPSDevice {
    fn drop(&mut self) {
        println!("Closing port {}", self.port_name);
        let _ = self.close();
    }
}

impl PartialEq for GPSDevice {
    fn eq(&self, other: &Self) -> bool {
        self.port_name == other.port_name
            && self.vid == other.vid
            && self.pid == other.pid
    }
}

const UBLOX_VID: u16 = 0x1546;
const UBLOX_PIDS: [u16; 2] = [0x01A8, 0x01A7];
const UBLOX_DEFAULT_BAUD: u32 = 115200;

impl GPSDevice {
    /// todo!()
    pub fn find_all() -> Result<Vec<Self>> {
        let gps_devices: Vec<Self> = serialport::available_ports()?
            .into_iter()
            .filter_map(|p| match &p.port_type {
                SerialPortType::UsbPort(upi) => {
                    if upi.vid == UBLOX_VID && UBLOX_PIDS.contains(&upi.pid) {
                        Some(Self {
                            port_name: p.port_name,
                            vid: upi.vid,
                            pid: upi.pid,
                            baud_rate: UBLOX_DEFAULT_BAUD,
                            port_handle: std::cell::RefCell::new(None),
                        })
                    }else {
                        None
                    }
                }
                _ => None,
            })
            .collect();
        Ok(gps_devices)
    }

    /// todo!()
    pub fn find_first() -> Result<Self> {
        let ports = Self::find_all()?;
        if ports.len() == 0 {
            Err(Error::InvalidDevice("No GPS Serial Ports found!".into()))
        } else {
            // Move instead of clone
            // https://github.com/rust-lang/rust-clippy/issues/5044
            Ok(ports.into_iter().nth(0).unwrap())
        }
    }

    pub fn open(&self) -> Result<()> {
        // Create the serial port and open it.
        *self.port_handle.borrow_mut() = Some(serialport::new(&self.port_name, self.baud_rate)
            .timeout(Duration::from_millis(100))
            .open()
            .map_err(Error::SerialError)
            .expect(format!("Failed to open port {}.", &self.port_name).as_str()));
        // Create a buffer and read the port.
        let mut buffer: Vec<u8> = vec![0; 1000];
        loop {
            match &mut *self.port_handle.borrow_mut() {
                Some(port) => {
                    match port.read(buffer.as_mut_slice()) {
                        // This reader has reached its "end of file" and will likely no longer be able to produce bytes.
                        Ok(size) if size == 0 => break,
                        // Successfully read some bytes
                        Ok(size) => {
                            let line = String::from_utf8(buffer[..size].to_vec()).unwrap();
                            let line = line.strip_suffix("\r\n").unwrap();
                            println!("{line}");
                        },
                        // Nothing to read, try again later
                        Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {},
                        // An error of the ErrorKind::Interrupted kind is non-fatal and the read operation should be retried if there is nothing else to do.
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                        // Fatal, this will happen when device is disconnected.
                        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                            break;
                        },
                        // Uncaught errors, panic!
                        Err(e) => {
                            panic!("Error: {e}");
                        },
                    }
                },
                None => todo!(),
            };
        }
        Ok(())
    }

    pub fn close(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use nmea_parser::NmeaParser;

    use super::*;

    #[test]
    fn test_find_gps_serial_port() -> Result<()> {
        let gps_devices = GPSDevice::find_all()?;
        println!("All GPS Devices: {gps_devices:#X?}");
        let gps_device: GPSDevice = GPSDevice::find_first().expect("Expected at least one device!");
        println!("{gps_device:#X?}");
        assert_eq!(gps_devices[0], gps_device);
        Ok(())
    }

    #[test]
    fn test() {
        let gps_device: GPSDevice = GPSDevice::find_first().expect("Expected at least one device!");
        gps_device.open().unwrap();
        gps_device.close().unwrap();
    }
}
