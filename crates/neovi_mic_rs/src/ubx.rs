use nom::{bytes::complete::take, error::ParseError, number::complete::be_u8};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug)]
pub enum Error {
    MalformedHeader(String),
    InvalidChecksum,
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::MalformedHeader(s) => write!(f, "Malformed GPS ubx header: {:#?}", s),
            Error::InvalidChecksum => write!(f, "Invalid Checksum"),
        }
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::MalformedHeader(value.to_string())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::MalformedHeader(value)
    }
}

impl<I> ParseError<I> for Error {
    fn from_error_kind(_input: I, _kind: nom::error::ErrorKind) -> Self {
        Self::MalformedHeader("Nom error".to_string())
    }

    fn append(_input: I, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<E> From<nom::Err<E>> for Error {
    fn from(value: nom::Err<E>) -> Self {
        match value {
            nom::Err::Error(_) => Self::MalformedHeader("nom Error".to_string()),
            nom::Err::Failure(_) => Self::MalformedHeader("nom Failure".to_string()),
            nom::Err::Incomplete(_) => Self::MalformedHeader("Incomplete data".to_string()),
        }
    }
}

// 25.1 Structure Packing
// Values are placed in an order that structure packing is not a problem. This means that 2Byte values shall start
// on offsets which are a multiple of 2, 4-byte values shall start at a multiple of 4, and so on. This can easily be
// achieved by placing the largest values first in the Message payload (e.g. R8), and ending with the smallest (i.e.
// one-byters such as U1) values.

// All multi-byte values are ordered in Little Endian format, unless otherwise indicated.
// All floating point values are transmitted in IEEE754 single or double precision.

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct PacketHeader {
    /// Every Message starts with 2 Bytes: 0xB5 0x62
    pub header: [u8; 2],
    /// Class field. The Class defines the basic subset of the message.
    pub class: ClassField,
    pub id: u8,
    /// length is defined as being the length of the payload, only. It does not
    /// include Sync Chars, Length Field, Class, ID or CRC fields.
    /// The number format of the length field is an
    /// unsigned 16-Bit integer in Little Endian Format.
    //pub length: u16,
    pub payload: Vec<u8>,
    /// CK_A and CK_B is a 16 Bit checksum. 
    /// The checksum algorithm used is the 8-Bit Fletcher Algorithm, 
    /// which is used in the TCP standard (RFC 1145).
    pub ck_a: u8,
    /// CK_A and CK_B is a 16 Bit checksum. 
    /// The checksum algorithm used is the 8-Bit Fletcher Algorithm, 
    /// which is used in the TCP standard (RFC 1145).
    pub ck_b: u8,
}

/// Ubx header is always 0xB5, 0x62 and the first two bytes.
const HEADER_SIGNATURE: [u8; 2] = [0xB5, 0x62];

impl PacketHeader {
    pub fn from_bytes(input: &[u8]) -> Result<Self> {
        let (input, header) = take::<usize, &[u8], Error>(2usize)(input)?;
        if header != HEADER_SIGNATURE {
            return Err(Error::MalformedHeader(
                "Header signature is not of expected values".to_string(),
            ));
        }
        let (input, class) = be_u8::<&[u8], Error>(input)?;
        let (input, id) = be_u8::<&[u8], Error>(input)?;
        let (input, length) = be_u8::<&[u8], Error>(input)?;
        let (input, payload) = take::<usize, &[u8], Error>(length as usize)(input)?;
        let (input, ck_a) = be_u8::<&[u8], Error>(input)?;
        let (input, ck_b) = be_u8::<&[u8], Error>(input)?;

        Ok(Self {
            header: header.try_into().unwrap(),
            class: ClassField::try_from(class)?,
            id,
            payload: payload.to_vec(),
            ck_a,
            ck_b,
        })
    }

    /// Convert the packet data to raw bytes, includes checksum.
    pub fn data(&self, include_checksum: bool) -> Vec<u8> {
        let mut bytes = vec![
            self.header[0],
            self.header[1],
            self.class as u8,
            self.id,
            self.payload.len() as u8,
        ];
        // add the payload bytes individually, I don't know of an
        // easy way to append a vec to another vec init above.
        for byte in &self.payload {
            bytes.push(*byte);
        }
        // include the checksum in the bytes, this is useful for
        // calculating the checksum, for example.
        if include_checksum {
            bytes.push(self.ck_a);
            bytes.push(self.ck_b);
        }
        bytes
    }

    /// Calculate the checksum and return it as a tuple (ck_a, ck_b).
    fn checksum(&self) -> (u8, u8) {
        // The checksum is calculated over the packet, starting and 
        // including the CLASS field, up until, but excluding, the
        // Checksum Field
        let bytes= self.data(false);
        let mut ck_a: u8 = 0;
        let mut ck_b: u8 = 0;
        for byte in bytes {
            ck_a = ck_a.wrapping_add(byte);
            ck_b = ck_b.wrapping_add(ck_a);
        }
        (ck_a, ck_b)
    }

    fn verify_checksum(&self) -> Result<()> {
        let checksum = self.checksum();
        if checksum.0 != self.ck_a || checksum.1 != self.ck_b {
            return Err(Error::InvalidChecksum);
        }
        Ok(())
    }
}

/// 24 UBX Class IDs
/// A Class is a grouping of messages which are related to each other.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[repr(u8)]
pub enum ClassField {
    UNK = 0x00,
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

impl TryFrom<u8> for ClassField {
    type Error = String;

    fn try_from(value: u8) -> std::result::Result<Self, String> {
        match value {
            x if x == Self::NAV as u8 => Ok(Self::NAV),
            x if x == Self::RXM as u8 => Ok(Self::RXM),
            x if x == Self::INF as u8 => Ok(Self::INF),
            x if x == Self::ACK as u8 => Ok(Self::ACK),
            x if x == Self::CFG as u8 => Ok(Self::CFG),
            x if x == Self::MON as u8 => Ok(Self::MON),
            x if x == Self::AID as u8 => Ok(Self::AID),
            x if x == Self::TIM as u8 => Ok(Self::TIM),
            x if x == Self::ESF as u8 => Ok(Self::ESF),
            _ => Err(format!("Invalid ubx class field: {value}!").to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use nom::AsBytes;

    use super::*;

    fn generate_header() -> PacketHeader {
        let raw_bytes = [0xB5, 0x62, 0x05, 0x01, 0x02, 0x05, 0x01, 0x25, 0x6D];
        let header = PacketHeader::from_bytes(&raw_bytes).unwrap();
        assert_eq!(
            header,
            PacketHeader {
                header: [0xB5, 0x62],
                class: ClassField::try_from(0x05).unwrap(),
                id: 0x1,
                payload: vec![0x05, 0x01],
                ck_a: 0x25,
                ck_b: 0x6D,
            }
        );
        assert_eq!(raw_bytes, header.data(true).as_bytes());
        header
    }

    #[test]
    fn test_packet_header() {
        generate_header();
    }

    #[test]
    fn test_checksum() {
        let header = generate_header();
        let checksum = header.checksum();
        assert_eq!(header.ck_a, checksum.0);
        assert_eq!(header.ck_b, checksum.1);
        header.verify_checksum().unwrap();
    }

    #[test]
    fn test_bad_checksum() {
        let mut header = generate_header();
        // lets corrupt the checksum
        header.ck_a = 0xDE;
        header.ck_b = 0xAD;
        assert!(header.verify_checksum().is_err());
    }

    #[test]
    fn test_bad_data_checksum() {
        let mut header = generate_header();
        // lets corrupt the data
        header.payload = vec![0,1,2,3,4,5,6,7];
        assert!(header.verify_checksum().is_err());
    }
}
