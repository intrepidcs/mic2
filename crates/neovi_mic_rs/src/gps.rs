use serialport::{self, SerialPortType, SerialPortInfo, UsbPortInfo, ErrorKind};
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

pub fn find_gps_serial_ports() -> Result<Vec<SerialPortInfo>> {
    let ports: Vec<SerialPortInfo> = serialport::available_ports()?;
    let ports = ports.into_iter().filter(|p| {
        match &p.port_type {
            SerialPortType::UsbPort(upi) => {
                // If we don't have a serial number, nothing to match here
                if upi.serial_number.is_none() {
                    false
                } else if upi.serial_number.as_ref().unwrap().starts_with("MC") {
                    true
                } else {
                    false
                }
            },
            _ => false
        }
    }).collect();
    Ok(ports)
}

pub fn find_first_gps_serial_port() -> Result<SerialPortInfo> {
    let ports = find_gps_serial_ports()?;
    if ports.len() == 0 {
        Err(Error::InvalidDevice("No GPS Serial Ports found!".into()))
    } else {
        // Move instead of clone
        // https://github.com/rust-lang/rust-clippy/issues/5044
        Ok(ports.into_iter().nth(0).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_gps_serial_port() {
        let port: SerialPortInfo = find_first_gps_serial_port().expect("Expected at least one device!");
        println!("{port:#?}")
    }
}
