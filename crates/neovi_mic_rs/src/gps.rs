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

#[derive(Debug)]
pub struct GPSDevice {
    /// Port name string similar to "/dev/ttyACM0"
    pub port_name: String,
    /// USB Vendor ID of the GPS
    pub vid: u16,
    /// USB product ID of the GPS
    pub pid: u16,
    /// baudrate of the port, typically 115200
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
                            baud_rate: 115200,
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
        let mut port = serialport::new(&gps_device.port_name, gps_device.baud_rate)
            .timeout(Duration::from_millis(10))
            .open()
            .expect("Failed to open port");
        let mut serial_buf: Vec<u8> = vec![0; 1000];
        println!(
            "Receiving data on {} at {} baud:",
            &gps_device.port_name, &gps_device.baud_rate
        );
        loop {
            match port.read(serial_buf.as_mut_slice()) {
                Ok(t) => {
                    let mut parser = NmeaParser::new();
                    let nmea_string = String::from_utf8(serial_buf[..t].to_vec()).unwrap();
                    let nmea_string = nmea_string.strip_suffix("\r\n").unwrap();
                    println!("{}", nmea_string);
                    //let nmea_sentence = parser.parse_sentence(nmea_string);
                    //println!("{:?}", nmea_sentence)
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
                Err(e) => eprintln!("{:?}", e),
            }
        }
    }
}
