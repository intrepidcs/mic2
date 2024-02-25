use nom::{bytes::complete::take, number::complete::be_u8, IResult};
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
            Self::InvalidChecksum => write!(f, "Invalid Checksum"),
        }
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::MalformedHeader(value)
    }
}

impl<T> From<nom::Err<T>> for Error {
    fn from(value: nom::Err<T>) -> Self {
        match value {
            nom::Err::Error(_) => {
                Self::MalformedHeader("The parser had a recoverable error...".to_string())
            }
            nom::Err::Failure(_) => {
                Self::MalformedHeader("The parser had an unrecoverable error".to_string())
            }
            nom::Err::Incomplete(_) => {
                Self::MalformedHeader("There was not enough data".to_string())
            }
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
    /// Take bytes and convert to [u8; 2]
    fn be_u8(input: &[u8]) -> IResult<&[u8], u8> {
        let (input, value) = be_u8(input)?;
        Ok((input, value.try_into().unwrap()))
    }

    /// Take length bytes and convert to Vec<u8>
    fn take_len(input: &[u8], length: usize) -> IResult<&[u8], Vec<u8>> {
        let (input, value) = take(length)(input)?;
        Ok((input, value.to_vec()))
    }

    // Take 2 bytes and convert to [u8; 2]
    fn take_2(input: &[u8]) -> IResult<&[u8], [u8; 2]> {
        let (input, value) = take(2usize)(input)?;
        Ok((input, value.try_into().unwrap()))
    }

    pub fn from_bytes(input: &[u8]) -> Result<Self> {
        let (input, header) = Self::take_2(input)?;
        if header != HEADER_SIGNATURE {
            return Err(Error::MalformedHeader(
                "Header signature is not of expected values".to_string(),
            ));
        }
        let (input, class) = Self::be_u8(input)?;
        let (input, id) = Self::be_u8(input)?;
        let (input, length) = Self::be_u8(input)?;
        let (input, payload) = Self::take_len(input, length as usize)?;
        let (input, ck_a) = Self::be_u8(input)?;
        let (_input, ck_b) = Self::be_u8(input)?;

        Ok(Self {
            header: header.try_into().unwrap(),
            class: ClassField::try_from(class)?,
            id,
            payload: payload,
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
#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq)]
#[repr(u8)]
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
            _ => {
                let msg = format!("Invalid ubx class field: {value}!");
                Err(msg)
            }
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
    fn test_valid_packet_header_from_bytes() {
        let raw_bytes = [
            HEADER_SIGNATURE[0],
            HEADER_SIGNATURE[1],
            ClassField::ACK as u8,
            0x01, // id
            0x02, // payload length
            0x05, // payload
            0x01, // payload
            0x0,  // ck_a
            0x0,  // ck_b
        ];
        let header = PacketHeader::from_bytes(&raw_bytes).unwrap();
        assert_eq!(
            header,
            PacketHeader {
                header: HEADER_SIGNATURE,
                class: ClassField::ACK,
                id: 0x1,
                payload: vec![0x05, 0x01],
                ck_a: 0x0,
                ck_b: 0x0,
            }
        );
    }

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
    
    fn test_invalid_packet_header_from_bytes() {
        let raw_bytes = [0xA5, 0x62, 0x05, 0x01, 0x02, 0x05, 0x01, 0x0, 0x0];
        assert!(PacketHeader::from_bytes(&raw_bytes).is_err());
    }

    #[test]
    fn test_bad_data_checksum() {
        let mut header = generate_header();
        // lets corrupt the data
        header.payload = vec![0,1,2,3,4,5,6,7];
        assert!(header.verify_checksum().is_err());
    }

    fn test_class_field_values() {
        // Make sure all valid values pass
        assert_eq!(ClassField::NAV as u8, 0x01);
        assert_eq!(ClassField::RXM as u8, 0x02);
        assert_eq!(ClassField::INF as u8, 0x04);
        assert_eq!(ClassField::ACK as u8, 0x05);
        assert_eq!(ClassField::CFG as u8, 0x06);
        assert_eq!(ClassField::MON as u8, 0x0A);
        assert_eq!(ClassField::AID as u8, 0x0B);
        assert_eq!(ClassField::TIM as u8, 0x0D);
        assert_eq!(ClassField::ESF as u8, 0x10);

        // Make sure all valid values pass
        ClassField::try_from(0x01).unwrap();
        ClassField::try_from(0x02).unwrap();
        ClassField::try_from(0x04).unwrap();
        ClassField::try_from(0x05).unwrap();
        ClassField::try_from(0x06).unwrap();
        ClassField::try_from(0x0A).unwrap();
        ClassField::try_from(0x0B).unwrap();
        ClassField::try_from(0x0D).unwrap();
        ClassField::try_from(0x10).unwrap();

        // Make sure all invalid values fail
        for i in 0x11..0xFF {
            assert!(ClassField::try_from(i).is_err());
        }
    }
}
