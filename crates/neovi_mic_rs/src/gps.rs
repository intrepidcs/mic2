use serialport::{self, ErrorKind, SerialPortInfo, SerialPortType};
use nmea_parser::NmeaParser;
use crate::types::{Error, Result};

impl From<serialport::Error> for Error {
    fn from(value: serialport::Error) -> Self {
        match &value.kind {
            ErrorKind::NoDevice => Error::InvalidDevice("No Serial Ports found!".into()),
            _ => Error::SerialError(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GPSDevice {
    /// Port name string similar to "/dev/ttyACM0"
    pub port_name: String,
    /// USB Vendor ID of the GPS
    pub vid: u16,
    /// USB product ID of the GPS
    pub pid: u16,
    /// baudrate of the port, typically 115200
    baud_rate: u32,
}

impl GPSDevice {
    /// todo!()
    pub fn find_all() -> Result<Vec<Self>> {
        let ports: Vec<SerialPortInfo> = serialport::available_ports()?;
        let ports: Vec<SerialPortInfo> = ports
            .into_iter()
            .filter(|p| match &p.port_type {
                SerialPortType::UsbPort(upi) => {
                    if upi.vid == 0x1546 && upi.pid == 0x01A8 {
                        true
                    } else if upi.vid == 0x1546 && upi.pid == 0x01A7 {
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            })
            .collect();
        let mut gps_devices = Vec::new();
        for port in &ports {

            let upi = match &port.port_type {
                SerialPortType::UsbPort(upi) => upi,
                _ => panic!("BUG! We shouldn't be here, we should have filtered out anything else."),
            };
            gps_devices.push(Self {
                port_name: port.port_name.to_string(),
                vid: upi.vid,
                pid: upi.pid,
                baud_rate: 115200,
            });
        }
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

    pub fn start(&self) -> Result<()> {
        let port = serialport::new(&self.port_name, self.baud_rate);
        let asdf = port.open()?;

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::{time::Duration, io::Write};

    use super::*;

    #[test]
    fn test_find_gps_serial_port() -> Result<()> {
        let gps_devices = GPSDevice::find_all()?;
        println!("All GPS Devices: {gps_devices:#X?}");
        let gps_device: GPSDevice =
            GPSDevice::find_first().expect("Expected at least one device!");
        println!("{gps_device:#X?}");
        assert_eq!(gps_devices[0], gps_device);
        Ok(())
    }

    #[test]
    fn test() {
        let gps_device: GPSDevice =
            GPSDevice::find_first().expect("Expected at least one device!");
        let mut port = serialport::new(&gps_device.port_name, gps_device.baud_rate)
        .timeout(Duration::from_millis(10))
        .open().expect("Failed to open port");
        let mut serial_buf: Vec<u8> = vec![0; 1000];
        println!("Receiving data on {} at {} baud:", &gps_device.port_name, &gps_device.baud_rate);
        loop {
            match port.read(serial_buf.as_mut_slice()) {
                Ok(t) => {
                    let mut parser = NmeaParser::new();
                    let nmea_string = String::from_utf8(serial_buf[..t].to_vec()).unwrap();
                    let nmea_string = nmea_string.strip_suffix("\r\n").unwrap();
                    let nmea_sentence = parser.parse_sentence(nmea_string);
                    println!("{:?}", nmea_sentence)
                },
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
                Err(e) => eprintln!("{:?}", e),
            }
        }
    }
    
}
