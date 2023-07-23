use core::panic;

use serialport::{self, SerialPortType, UsbPortInfo};
use nmea_parser::NmeaParser;

pub fn find_gps_serial_port() -> String {
    let ports = serialport::available_ports().expect("No Serial Ports found!");
    let mut com_port = None;
    for port in ports {
        println!("{:?}", port.port_type);
        com_port = match &port.port_type {
            SerialPortType::UsbPort(info) => {
                if info.serial_number.as_ref().unwrap().starts_with("MC") {
                    Some(port.port_name)
                } else {
                    None
                }
            },
            _ => None
        };
        if com_port.is_some() {
            break;
        }
    }
    com_port.expect("Failed to find com port to open!")
}

/*
fn main() {
    let com_port = find_gps_serial_port();
    println!("Com port: {}", com_port);

    let port = serialport::new(com_port, 57_600u32).open().expect("Failed to open {com_port}");
    let parser = NmeaParser::new();
    println!("Hello, world!");
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
