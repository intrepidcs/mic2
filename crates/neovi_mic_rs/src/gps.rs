use std::{borrow::BorrowMut, time::Duration};

use crate::{
    nmea::{
        sentence::NMEASentence,
        types::{GPSDMS, GPSInfo, GpsNavigationStatus, NMEAError, NMEASentenceType},
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
    NMEAPartialStart(String),
    NMEAPartial(String),
    NMEAPartialEnd(String),
}

impl GPSPacket {
    pub fn new(bytes: &[u8]) -> Vec<Self> {
        let mut packets = Vec::new();
        for packet in NMEASentence::from_bytes(&bytes) {
            packets.push(match packet {
                Ok(ns) => match ns.data() {
                    Ok(nst) => GPSPacket::NMEA(nst),
                    Err(e) => GPSPacket::NMEAUnsupported(ns.inner, e),
                },
                Err(NMEAError::PartialStart(s)) => Self::NMEAPartialStart(s),
                Err(NMEAError::Partial(s)) => Self::NMEAPartial(s),
                Err(NMEAError::PartialEnd(s)) => Self::NMEAPartialEnd(s),
                Err(NMEAError::InvalidMode(s)) => Self::Unsupported(bytes.to_vec(), s),
                Err(NMEAError::InvalidData(s)) => {
                    match ubx::PacketHeader::from_bytes(&bytes) {
                        Ok(p) => Self::Ubx(p),
                        Err(e) => Self::Unsupported(bytes.to_vec(), e.to_string()),
                    }
                },
            });
        }
        packets
    }
}

#[derive(Debug, Default)]
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
    gps_info: std::sync::Arc<std::sync::RwLock<GPSInfo>>,
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
    pub fn find(vid: &u16, pid: &u16) -> Option<Self> {
        for device in Self::find_all().unwrap() {
            if device.vid == *vid && device.pid == *pid {
                return Some(device);
            }
        }
        None
    }
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
                            gps_info: std::sync::Arc::new(std::sync::RwLock::new(
                                GPSInfo::default(),
                            )),
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

            // 32.10.29.1 Reset receiver / Clear backup data structures
            // Payload: Hot Start (0x00), Controlled software reset (0x01), reserved1 (0x00)
            port.write_all(
                ubx::PacketHeader::new(ubx::ClassField::CFG, 0x04, vec![0x0, 0x01, 0], true)
                    .data(true)
                    .as_slice(),
            )
            .unwrap();

            // Disable all NEMA messages
            // 31.1.9 Messages overview
            for i in vec![
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0D, 0x0E, 0x0F,
                0x40, 0x41, 0x42, 0x43, 0x44,
            ] {
                port.write_all(
                    ubx::PacketHeader::new(ubx::ClassField::CFG, 0x01, vec![0xF0, i, 0], true)
                        .data(true)
                        .as_slice(),
                )
                .unwrap();
            }
            // Enable UBX messages UBX,00 UBX,03 UBX,04
            // 19 NMEA Messages Overview
            for i in [0u8, 3, 4] {
                let cfg_msg_pkt = ubx::PacketHeader::new(
                    ubx::ClassField::CFG,
                    0x01,
                    vec![0xF1, i, 1], //0, 0, 0, 1, 0, 0],
                    true,
                );
                let data = cfg_msg_pkt.data(true);
                port.write_all(data.as_slice()).unwrap();
                //println!("Sent CFG message: {:02X?}", data);
                // TODO: Verify we got ACK messages from the GPS
            }

            let mut partial_sentence = String::new();
            let mut partial_complete = false;
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
                        let data = if partial_complete {
                            partial_complete = false;
                            partial_sentence.as_str().as_bytes()
                        } else {
                            &buffer[..size]
                        };
                        let packets = GPSPacket::new(&data);
                        // Parse the packet
                        for packet in packets {
                            match &packet {
                                GPSPacket::NMEA(nmea) => match nmea {
                                    NMEASentenceType::PUBX00(data) => {
                                        gps_info.write().unwrap().update_from_nmea_sentence(nmea);
                                    }
                                    NMEASentenceType::PUBX03(data) => {
                                        println!("PUBX03: {data:?}");
                                        gps_info.write().unwrap().update_from_nmea_sentence(nmea);
                                    }
                                    NMEASentenceType::PUBX04(data) => {
                                        gps_info.write().unwrap().update_from_nmea_sentence(nmea);
                                    }
                                    _ => panic!("Unsupported sentence: {nmea:?}"),
                                },
                                GPSPacket::Ubx(packet) => println!("Received: {:#?}", packet),
                                GPSPacket::Unsupported(data, e) => {
                                    println!("Unsupported: {data:?} {e}")
                                }
                                GPSPacket::NMEAUnsupported(data, e) => {
                                    println!("Unsupported NMEA: {data:?} {e:?}")
                                }
                                GPSPacket::NMEAPartialStart(s) => {
                                    partial_sentence = s.to_owned();
                                    partial_complete = false;
                                },
                                GPSPacket::NMEAPartial(s) => partial_sentence.push_str(s),
                                GPSPacket::NMEAPartialEnd(s) => {
                                    partial_sentence.push_str(s);
                                    partial_complete = true;
                                },
                            }
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

    /// Close the GPS connection.
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

    /// Returns the current GPS Info. See [GPSInfo] for more info. Port should be open first.
    pub fn get_info(&self) -> GPSInfo {
        self.gps_info.read().unwrap().clone()
    }

    /// Returns true if the GPS has a fix. False if it does not.
    pub fn has_lock(&self) -> bool {
        match &self.gps_info.read().unwrap().nav_stat {
            Some(GpsNavigationStatus::NoFix) => false,
            _ => true,
        }
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
            println!("{info:?}");
        }
        gps_device.close().unwrap();
    }
}
