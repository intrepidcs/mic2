use std::{
    borrow::BorrowMut,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, RwLock,
    },
    time::Duration,
};

use crate::{
    nmea::{
        sentence::NMEASentence,
        types::{GPSInfo, GpsNavigationStatus, NMEAError, NMEASentenceType},
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

#[allow(clippy::upper_case_acronyms)]
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
        for packet in NMEASentence::from_bytes(bytes) {
            packets.push(match packet {
                Ok(ns) => match ns.data() {
                    Ok(nst) => GPSPacket::NMEA(nst),
                    Err(e) => GPSPacket::NMEAUnsupported(ns.inner, e),
                },
                Err(NMEAError::PartialStart(s)) => Self::NMEAPartialStart(s),
                Err(NMEAError::Partial(s)) => Self::NMEAPartial(s),
                Err(NMEAError::PartialEnd(s)) => Self::NMEAPartialEnd(s),
                Err(NMEAError::InvalidMode(s)) => Self::Unsupported(bytes.to_vec(), s),
                Err(NMEAError::InvalidData(_)) => match ubx::PacketHeader::from_bytes(bytes) {
                    Ok(p) => Self::Ubx(p),
                    Err(e) => Self::Unsupported(bytes.to_vec(), e.to_string()),
                },
            });
        }
        packets
    }
}

#[derive(Debug, Default, Clone)]
pub struct GPSDevice {
    /// Port name string similar to "/dev/ttyACM0"
    pub port_name: String,
    /// USB Vendor ID of the GPS
    pub vid: u16,
    /// USB product ID of the GPS
    pub pid: u16,
    /// baudrate of the port, typically UBLOX_DEFAULT_BAUD
    baud_rate: u32,
    /// If set to true, it tells the thread to shutdown.
    shutdown_thread: Arc<AtomicBool>,
    /// thread sets this to true when the thread is running, otherwise false.
    thread_running: Arc<AtomicBool>,
    /// Whether the port is open or not
    is_open: Arc<AtomicBool>,
    gps_info: Arc<RwLock<GPSInfo>>,
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
        match Self::find_all() {
            Ok(devices) => devices.into_iter().find(|d| d.vid == *vid && d.pid == *pid),
            Err(_e) => None,
        }
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
                            //thread: std::cell::RefCell::new(None),
                            shutdown_thread: std::sync::Arc::new(
                                std::sync::atomic::AtomicBool::new(false),
                            ),
                            thread_running: std::sync::Arc::new(
                                std::sync::atomic::AtomicBool::new(false),
                            ),
                            is_open: Arc::new(AtomicBool::new(false)),
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
        if ports.is_empty() {
            Err(Error::InvalidDevice("No GPS Serial Ports found!".into()))
        } else {
            // Move instead of clone
            // https://github.com/rust-lang/rust-clippy/issues/5044
            Ok(ports.into_iter().nth(0).unwrap())
        }
    }

    pub fn open(&self) -> Result<bool> {
        // Nothing to do if already open
        if self.thread_running.load(Ordering::Relaxed) {
            return Ok(true);
        }
        // Prepare the thread variables
        let port_name = self.port_name.clone();
        let baud_rate = self.baud_rate;
        let shutdown_thread = self.shutdown_thread.clone();
        shutdown_thread.store(false, Ordering::SeqCst);
        let thread_running = self.thread_running.clone();
        thread_running.store(false, Ordering::SeqCst);
        let is_open = self.is_open.clone();
        is_open.store(false, Ordering::SeqCst);
        let gps_info = self.gps_info.clone();
        // create the thread
        let (tx, rx) = mpsc::channel();
        let _thread = std::thread::spawn(move || {
            thread_running.store(true, Ordering::SeqCst);
            is_open.store(false, Ordering::SeqCst);
            // We notify the condvar that the value has changed.
            // Open the port
            println!("Opening port {}", port_name);
            let mut port = serialport::new(&port_name, baud_rate)
                .timeout(Duration::from_millis(10))
                .open()
                .map_err(Error::SerialError)
                .unwrap_or_else(|_| panic!("Failed to open port {}.", &port_name));
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
            for i in [
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
            is_open.store(true, Ordering::Relaxed);
            tx.send(()).unwrap();
            loop {
                // Detect if we should shutdown
                if shutdown_thread.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                // read the port
                match port.read(buffer.as_mut_slice()) {
                    // This reader has reached its "end of file" and will likely no longer be able to produce bytes.
                    Ok(0) => break,
                    // Successfully read some bytes
                    Ok(size) => {
                        let data = if partial_complete {
                            partial_complete = false;
                            partial_sentence.as_bytes()
                        } else {
                            &buffer[..size]
                        };
                        let packets = GPSPacket::new(data);
                        // Parse the packet
                        for packet in packets {
                            match &packet {
                                GPSPacket::NMEA(nmea) => match nmea {
                                    NMEASentenceType::PUBX00(_) => {
                                        gps_info.write().unwrap().update_from_nmea_sentence(nmea);
                                    }
                                    NMEASentenceType::PUBX03(_) => {
                                        gps_info.write().unwrap().update_from_nmea_sentence(nmea);
                                    }
                                    NMEASentenceType::PUBX04(_) => {
                                        gps_info.write().unwrap().update_from_nmea_sentence(nmea);
                                    }
                                    _ => panic!("Unsupported sentence: {nmea:?}"),
                                },
                                GPSPacket::Ubx(_packet) => {} //println!("Received: {:#?}", packet),
                                GPSPacket::Unsupported(data, e) => {
                                    println!("Unsupported: {data:?} {e}")
                                }
                                GPSPacket::NMEAUnsupported(data, e) => {
                                    println!("Unsupported NMEA: {data:?} {e:?}")
                                }
                                GPSPacket::NMEAPartialStart(s) => {
                                    s.clone_into(&mut partial_sentence);
                                    partial_complete = false;
                                }
                                GPSPacket::NMEAPartial(s) => partial_sentence.push_str(s),
                                GPSPacket::NMEAPartialEnd(s) => {
                                    partial_sentence.push_str(s);
                                    partial_complete = true;
                                }
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
            is_open.store(false, Ordering::Relaxed);
            thread_running.store(false, Ordering::SeqCst);
            //tx.send(()).unwrap();
        });
        rx.recv().unwrap();
        Ok(self.is_open.load(std::sync::atomic::Ordering::Relaxed))
    }

    /// Close the GPS connection.
    pub fn close(&self) -> Result<()> {
        self.shutdown_thread
            .clone()
            .borrow_mut()
            .store(true, Ordering::Relaxed);
        while self.thread_running.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(3));
        }
        Ok(())
    }

    pub fn is_open(&self) -> bool {
        self.is_open.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Returns the current GPS Info. See [GPSInfo] for more info. Port should be open first.
    pub fn get_info(&self) -> Result<GPSInfo> {
        if !self.is_open() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Serial Port not open",
            )
            .into());
        }
        Ok(self.gps_info.read().unwrap().clone())
    }

    /// Returns true if the GPS has a fix. False if it does not.
    pub fn has_lock(&self) -> Result<bool> {
        if !self.is_open() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Serial Port not open",
            )
            .into());
        }
        match &self.gps_info.read().unwrap().nav_stat {
            Some(GpsNavigationStatus::NoFix) => Ok(false),
            Some(_) => Ok(true),
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
#[cfg(not(feature = "_skip-hil-testing"))]
mod tests_hil {
    use std::time::Duration;

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
        for _ in 0..1000 {
            std::thread::sleep(Duration::from_millis(1000));
            let info = gps_device.get_info().unwrap();
            println!(
                "Has Fix: {:?}\nSats: {:?}\nInfo: {:?}",
                gps_device.has_lock(),
                &info.satellites.len(),
                &info
            );
        }
        gps_device.close().unwrap();
    }
}
