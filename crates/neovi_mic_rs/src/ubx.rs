use std::fmt;
use serde::{Serialize, Deserialize};
use nom::{
    bytes::complete::take,
    number::complete::be_u8,
    error::ParseError,
};


#[derive(Debug)]
pub enum Error {
    MalformedHeader(String),
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::MalformedHeader(s) => write!(f, "Malformed GPS ubx header: {:#?}", s),
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
    pub ck_a: u8,
    pub ck_b: u8,
}

/// Ubx header is always 0x85, 0x62 and the first two bytes.
const HEADER_SIGNATURE: [u8; 2] = [0x85, 0x62];

impl PacketHeader {
    pub fn from_bytes(input: &[u8]) -> Result<Self> {
        let (input, header) = take::<usize, &[u8], Error>(2usize)(input)?;
        if header != HEADER_SIGNATURE {
            return Err(Error::MalformedHeader("Header signature is not of expected values".to_string()));
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
}


#[derive(Debug, Deserialize, Serialize, PartialEq)]
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
    use super::*;

    #[test]
    fn test_packet_header_from() {
        let raw_bytes = [0xB5, 0x62, 0x05, 0x01, 0x02, 0x05, 0x01, 0x0, 0x0];
        let header = PacketHeader::from_bytes(&raw_bytes).unwrap();
        assert_eq!(header, PacketHeader {
            header: [0xB5, 0x62],
            class: ClassField::try_from(0x05).unwrap(),
            id: 0x1,
            payload: vec![0x05, 0x01],
            ck_a: 0x0,
            ck_b: 0x0,
        });
    }
}
