use std::fmt;


#[derive(Debug, Display)]
pub enum Error {
    MalformedHeader(String),
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::MalformedHeader(s) => write!(f, "Malformed ubx header: {:#?}", s),
        }
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::MalformedHeader(value.to_string())
    }
}


// 25.1 Structure Packing
// Values are placed in an order that structure packing is not a problem. This means that 2Byte values shall start
// on offsets which are a multiple of 2, 4-byte values shall start at a multiple of 4, and so on. This can easily be
// achieved by placing the largest values first in the Message payload (e.g. R8), and ending with the smallest (i.e.
// one-byters such as U1) values.

// All multi-byte values are ordered in Little Endian format, unless otherwise indicated.
// All floating point values are transmitted in IEEE754 single or double precision.

#[derive(Clone, Default, Debug, PartialEq)]
#[repr(packed)]
struct Header {
    /// Every Message starts with 2 Bytes: 0xB5 0x62
    pub header: [u8; 2],
    /// Class field. The Class defines the basic subset of the message.
    pub class: u8,
    pub id: u8,
    /// ength is defined as being the length of the payload, only. It does not
    /// include Sync Chars, Length Field, Class, ID or CRC fields. 
    /// The number format of the length field is an
    /// unsigned 16-Bit integer in Little Endian Format.
    pub length: u16,
    pub payload: Vec<u8>,
    pub ck_a: u8,
    pub ck_b: u8,
}

const HEADER_TOTAL_MIN_SIZE: usize = 8;
const HEADER_MIN_SIZE: usize = 4;
const HEADER_SIGNATURE: [u8; 2] = [0x85, 0x62];

#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
enum HeaderOffset {
    Header = 0x0,
    Class = 0x2,
    ID = 0x3,
    Length = 0x4,
    Payload = 0x6,
}


impl Header {
    pub fn from(data: &[u8]) -> Result<Self> {
        if data.len() <= HEADER_TOTAL_MIN_SIZE {
            return Err(Error::MalformedHeader("Header size is too small".to_string()));
        }
        if data[0..2] != HEADER_SIGNATURE {
            return Err(Error::MalformedHeader("Header signature is not of expected values".to_string()));
        }
        let length: usize = data[HeaderOffset::Length.into()] + (data[HeaderOffset::Length.into()+1] << 8);
        let payload = &data[HeaderOffset::Payload as usize..HeaderOffset::Payload.into()+length];
        let s = Self {
            header: data[0..2].try_into().unwrap(),
            class: data[HeaderOffset::Class.into()],
            id: data[HeaderOffset::ID.into()],
            length: length,
            payload: payload.into(),
            ck_a: data[HeaderOffset::Payload as usize+length],
            ck_b: data[HeaderOffset::Payload as usize+length+1],
        };
        Ok(s)
    }
}

pub enum ClassField {
    /// Navigation Results: Position, Speed, Time, Acc, Heading, DOP, SVs used
    NAV = 0x01,
    /// Receiver Manager Messages: Satellite Status, RTC Status
    RXM = 0x02, 
    /// Information Messages: Printf-Style Messages, with IDs such as Error, Warning, Notice
    INF = 0x04,
    /// Ack/Nack Messages: as replies to CFG Input Messages
    ACK = 0x05,
    /// Configuration Input Messages: Set Dynamic Model, Set DOP Mask, Set Baud Rate, etc.
    CFG = 0x06,
    /// Monitoring Messages: Comunication Status, CPU Load, Stack Usage, Task Status
    MON = 0x0A,
    /// Timing Messages: Timepulse Output, Timemark Results
    AID = 0x0B,
    /// AssistNow Aiding Messages: Ephemeris, Almanac, other A-GPS data input
    TIM = 0x0D, 
    /// External Sensor Fusion Messages: External sensor measurements and status information
    ESF = 0x10, 
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_from() {
        let raw_bytes = [0xB5, 0x62, 0x05, 0x01, 0x02, 0x05, 0x01, 0x0, 0x0];
        let header = Header::from(raw_bytes);
        assert_eq!(header, Header {
            header: [0xB5, 0x62],
            class: 0x05,
            id: 0x1,
            length: 0x02,
            payload: [0x05, 0x01],
            ck_a: 0x0,
            ck_b: 0x0,
        });
    }
}