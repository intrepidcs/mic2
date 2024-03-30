use std::{borrow::BorrowMut, time::Duration};

use crate::{
    nmea::{
        sentence::NMEASentence,
        types::{GpsInfo, NMEAError, NMEASentenceType},
    },
    types::{Error, Result},
    ubx,
};
use serialport::{self, ErrorKind, SerialPortType};

impl From<serialport::Error> for Error {
    fn from(value: serialport::Error) -> Self {
        match &value.kind {
            ErrorKind::NoDevice => Error::InvalidDevice("No Serial Ports found!".into()),
            _ => Error::SerialError(value),
        }
    }
}

#[derive(Debug)]
enum GPSPacket {
    Ubx(ubx::PacketHeader),
    NMEA(NMEASentenceType),
    NMEAUnsupported(String, NMEAError),
    Unsupported(Vec<u8>, String),
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
    gps_info: std::sync::Arc<std::sync::RwLock<GpsInfo>>,
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
                            gps_info: std::sync::Arc::new(std::sync::RwLock::new(GpsInfo::default())),
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
        let gps_info = self.gps_info.clone();
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
                let cfg_msg_pkt = ubx::PacketHeader::new(
                    ubx::ClassField::CFG,
                    0x01,
                    vec![0xF1, i, 0, 0, 0, 1, 0, 0],
                    true,
                );
                let data = cfg_msg_pkt.data(true);
                port.write_all(data.as_slice()).unwrap();
                println!("Sent CFG message: {:02X?}", data);
                // TODO: Verify we got ACK messages from the GPS
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
                        let data = &buffer[..size];
                        // TODO: Clean this up, a lot of this should be moved to GPSPacket
                        // Check if this is a UBX message
                        let packet = match ubx::PacketHeader::from_bytes(&data) {
                            Ok(packet_header) => GPSPacket::Ubx(packet_header),
                            Err(_e) => {
                                // This is not a UBX message, more than likely its a NMEA statement
                                // TODO: Clean this up, a lot of this should be moved to NMEASentence
                                match String::from_utf8(data.to_vec()) {
                                    Ok(nmea_str) => match NMEASentence::from_bytes(&data) {
                                        Ok(ns) => match ns.data() {
                                            Ok(nst) => GPSPacket::NMEA(nst),
                                            Err(e) => GPSPacket::NMEAUnsupported(nmea_str, e),
                                        },
                                        Err(e) => GPSPacket::NMEAUnsupported(nmea_str, e),
                                    },
                                    Err(e) => {
                                        // We failed to convert the bytes to a string
                                        GPSPacket::Unsupported(data.to_vec(), e.to_string())
                                    }
                                }
                            }
                        };
                        // Parse the packet
                        match &packet {
                            GPSPacket::NMEA(nmea) => { match nmea {
                                NMEASentenceType::PUBX00(data) => {
                                    gps_info.write().unwrap().update_from_nmea_sentence(nmea);
                                }
                                NMEASentenceType::PUBX03(data) => {
                                    gps_info.write().unwrap().update_from_nmea_sentence(nmea);
                                }
                                NMEASentenceType::PUBX04(data) => {
                                    gps_info.write().unwrap().update_from_nmea_sentence(nmea);
                                }
                                _ => panic!("Unsupported sentence: {nmea:?}"),
                            }},
                            GPSPacket::Ubx(packet) => println!("Received: {:#?}", packet),
                            GPSPacket::Unsupported(data, e) => println!("Unsupported: {data:?} {e}"),
                            GPSPacket::NMEAUnsupported(data, e) => println!("Unsupported: {data:?} {e:?}"),
                        }
                        //println!("GPSInfo: {:?}", gps_info.read().unwrap());
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

    pub fn get_info(&self) -> GpsInfo {
        self.gps_info.read().unwrap().clone()
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
        for _ in 0..10 {
            std::thread::sleep(Duration::from_millis(1000));
            let info = gps_device.get_info();
            println!("{info:#?}");
        }
        gps_device.close().unwrap();
        
    }
}
