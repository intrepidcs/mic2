use std::{
    borrow::BorrowMut,
    fmt,
    time::Duration,
};

use crate::{
    types::{Error, Result},
    ubx,
};
use chrono::NaiveTime;
use serialport::{self, ErrorKind, SerialPortType};

impl From<serialport::Error> for Error {
    fn from(value: serialport::Error) -> Self {
        match &value.kind {
            ErrorKind::NoDevice => Error::InvalidDevice("No Serial Ports found!".into()),
            _ => Error::SerialError(value),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct GPSLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}

impl GPSLocation {
    pub fn new(latitude: f64, longitude: f64, altitude: f64) -> Self {
        Self {
            latitude,
            longitude,
            altitude,
        }
    }
}

impl fmt::Display for GPSLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Lat: {}, Long: {}, Alt: {}",
            self.latitude, self.longitude, self.altitude
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct GPSLock {
    pub locked: bool,
    pub accuracy: f64,
}

impl GPSLock {
    pub fn new(locked: bool, accuracy: f64) -> Self {
        Self { locked, accuracy }
    }
}

impl fmt::Display for GPSLock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Locked: {}, Accuracy: {}", self.locked, self.accuracy)
    }
}

#[derive(Debug)]
pub struct GPSData {
    pub location: GPSLocation,
    pub lock: GPSLock,
    pub time: NaiveTime,
}

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
    thread: std::cell::RefCell<Option<std::thread::JoinHandle<()>>>,
    shutdown_thread: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Drop for GPSDevice {
    fn drop(&mut self) {
        println!("Closing port {}", self.port_name);
        let _ = self.close();
    }
}

impl PartialEq for GPSDevice {
    fn eq(&self, other: &Self) -> bool {
        self.port_name == other.port_name && self.vid == other.vid && self.pid == other.pid
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
                            thread: std::cell::RefCell::new(None),
                            shutdown_thread: std::sync::Arc::new(
                                std::sync::atomic::AtomicBool::new(false),
                            ),
                        })
                    } else {
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
        // Nothing to do if already open
        if self.thread.borrow().is_some() {
            return Ok(());
        }
        // create the thread
        let port_name = self.port_name.clone();
        let baud_rate = self.baud_rate;
        let shutdown_thread = self.shutdown_thread.clone();
        *self.thread.borrow_mut() = Some(std::thread::spawn(move || {
            // Open the port
            println!("Opening port {}", port_name);
            let mut port = serialport::new(&port_name, baud_rate)
                .timeout(Duration::from_millis(10))
                .open()
                .map_err(Error::SerialError)
                .expect(format!("Failed to open port {}.", &port_name).as_str());
            let mut buffer: Vec<u8> = vec![0; 1000];

            // setup the port
            // Enable UBX messages UBX,00 UBX,03 UBX,04
            // 19 NMEA Messages Overview
            for i in [0u8, 3, 4] {
                let cfg_msg_pkt = ubx::PacketHeader::new(ubx::ClassField::CFG, 0x01, vec![0xF1, i, 0, 0, 0, 1, 0, 0], true);
                let data = cfg_msg_pkt.data(true);
                port.write_all(data.as_slice()).unwrap();
                println!("Sent CFG message: {:02X?}", data);
            }
            
            loop {
                // Detect if we should shutdown
                if shutdown_thread.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                // read the port
                match port.read(buffer.as_mut_slice()) {
                    // This reader has reached its "end of file" and will likely no longer be able to produce bytes.
                    Ok(size) if size == 0 => break,
                    // Successfully read some bytes
                    Ok(size) => {
                        if buffer[0] == 0xB5 {
                            let packet = ubx::PacketHeader::from_bytes(&buffer[..size]).unwrap();
                            let data = buffer[..size].to_vec();
                            println!("Received: {size}\t{:02X?} {packet:?}", data);
                            //println!("{packet:?}");
                        } else {
                            let line = match String::from_utf8(buffer[..size].to_vec()) {
                                Ok(l) => l,
                                Err(e) => {
                                    println!("Error: {e} {:02X?}", buffer);
                                    format!("Received: {size}\t{:02X?}", buffer).to_string()
                                }
                            };
                            let line = line.strip_suffix("\r\n").unwrap_or_default();
                            println!("Received: {size}\t{line}");
                        }
                    }
                    // Nothing to read, try again later
                    Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                    // An error of the ErrorKind::Interrupted kind is non-fatal and the read operation should be retried if there is nothing else to do.
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                    // Fatal, this will happen when device is disconnected.
                    Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                        break;
                    }
                    // Uncaught errors, panic!
                    Err(e) => {
                        panic!("Error: {e}");
                    }
                }
            }
        }));
        Ok(())
    }

    pub fn close(&self) -> Result<()> {
        // Nothing to do if already closed
        if self.thread.borrow().is_none() {
            return Ok(());
        }

        self.shutdown_thread
            .clone()
            .borrow_mut()
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.thread.borrow_mut().take().unwrap().join().unwrap();
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
        std::thread::sleep(Duration::from_millis(10000));
        gps_device.close().unwrap();
    }
}
