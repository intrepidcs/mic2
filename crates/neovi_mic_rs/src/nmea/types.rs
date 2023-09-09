// https://gpsd.gitlab.io/gpsd/NMEA.html

use chrono::NaiveTime;
use std::{
    borrow::Cow,
    fmt,
    num::{ParseFloatError, ParseIntError},
};

#[derive(Debug, Clone)]
pub enum NMEATalkerID {
    /// BeiDou (China) - $BD
    Beidou,
    /// Galileo Positioning System - $GA
    Galileo,
    /// GLONASS, according to IEIC 61162-1 - $GL
    Glonass,
    /// Combination of multiple satellite systems (NMEA 1083) - $GN
    Combination,
    /// Global Positioning System receiver - $GP
    Gps,
    /// NavIC, IRNSS (India) - $GI
    Navic,
    /// QZSS regional GPS augmentation system (Japan) - - $GQ
    Qzss,
    /// Unknown
    Unknown(String),
}

impl fmt::Display for NMEATalkerID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Beidou => write!(f, "BD"),
            Self::Galileo => write!(f, "GA"),
            Self::Glonass => write!(f, "GL"),
            Self::Combination => write!(f, "GN"),
            Self::Gps => write!(f, "GP"),
            Self::Navic => write!(f, "GI"),
            Self::Qzss => write!(f, "GQ"),
            Self::Gps => write!(f, "GP"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FAAModeType {
    Autonomous,
    Caution,
    Differential,
    Estimated,
    RTKFloat,
    ManualInput,
    DataNotValid,
    Precise,
    RTKInteger,
    Simulated,
    Unsafe,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NMEAError {
    InvalidData(String),
    InvalidMode(String),
}

impl std::convert::From<ParseIntError> for NMEAError {
    fn from(value: ParseIntError) -> Self {
        NMEAError::InvalidData(value.to_string())
    }
}

impl std::convert::From<ParseFloatError> for NMEAError {
    fn from(value: ParseFloatError) -> Self {
        NMEAError::InvalidData(value.to_string())
    }
}

/// In NMEA 2.3, several sentences (APB, BWC, BWR, GLL, RMA, RMB, RMC, VTG, WCV, and XTE) got
/// a new last field carrying the signal integrity information needed by the FAA. (The
/// values in the GGA mode field were extended to carry this information as well.)
#[derive(Clone, Debug, PartialEq)]
pub struct FAAMode {
    pub mode: String,
    pub mode_type: FAAModeType,
    pub description: String,
}

impl FAAMode {
    /// Creates a FAAMode based on character in NMEA sentence.
    pub fn from(mode: impl Into<&'static str>) -> Result<Self, NMEAError> {
        let mode_str = mode.into();
        match mode_str {
            "A" => Ok(Self::new_as_autonomous()),
            "C" => Ok(Self::new_as_caution()),
            "D" => Ok(Self::new_as_differential()),
            "E" => Ok(Self::new_as_estimated()),
            "F" => Ok(Self::new_as_rtk_float()),
            "M" => Ok(Self::new_as_manual_input()),
            "N" => Ok(Self::new_as_data_not_valid()),
            "P" => Ok(Self::new_as_precise()),
            "R" => Ok(Self::new_as_rtk_integer()),
            "S" => Ok(Self::new_as_simulated()),
            "U" => Ok(Self::new_as_unsafe()),
            _ => Err(NMEAError::InvalidMode(
                format!("Invalid mode '{:?}'", &mode_str).to_string(),
            )),
        }
    }

    /// Returns a FAAMode based on a FAAModeType enum.
    pub fn from_mode(mode_type: &FAAModeType) -> Self {
        match &mode_type {
            FAAModeType::Autonomous => Self::new_as_autonomous(),
            FAAModeType::Caution => Self::new_as_caution(),
            FAAModeType::Differential => Self::new_as_differential(),
            FAAModeType::Estimated => Self::new_as_estimated(),
            FAAModeType::RTKFloat => Self::new_as_rtk_float(),
            FAAModeType::ManualInput => Self::new_as_manual_input(),
            FAAModeType::DataNotValid => Self::new_as_data_not_valid(),
            FAAModeType::Precise => Self::new_as_precise(),
            FAAModeType::RTKInteger => Self::new_as_rtk_integer(),
            FAAModeType::Simulated => Self::new_as_simulated(),
            FAAModeType::Unsafe => Self::new_as_unsafe(),
            FAAModeType::Unknown(s) => Self::new_as_unknown(s),
        }
    }

    pub fn new_as_autonomous() -> Self {
        Self {
            mode: "A".to_string(),
            mode_type: FAAModeType::Autonomous,
            description: "Autonomous mode".to_string(),
        }
    }

    pub fn new_as_caution() -> Self {
        Self {
            mode: "C".to_string(),
            mode_type: FAAModeType::Caution,
            description: "Quectel Querk, \"Caution\" mode".to_string(),
        }
    }

    pub fn new_as_differential() -> Self {
        Self {
            mode: "D".to_string(),
            mode_type: FAAModeType::Differential,
            description: "Differential Mode".to_string(),
        }
    }

    pub fn new_as_estimated() -> Self {
        Self {
            mode: "E".to_string(),
            mode_type: FAAModeType::Estimated,
            description: "Estimated (dead-reckoning) mode".to_string(),
        }
    }

    pub fn new_as_rtk_float() -> Self {
        Self {
            mode: "F".to_string(),
            mode_type: FAAModeType::RTKFloat,
            description: "RTK Float mode".to_string(),
        }
    }

    pub fn new_as_manual_input() -> Self {
        Self {
            mode: "M".to_string(),
            mode_type: FAAModeType::ManualInput,
            description: "Manual Input Mode".to_string(),
        }
    }

    pub fn new_as_data_not_valid() -> Self {
        Self {
            mode: "N".to_string(),
            mode_type: FAAModeType::DataNotValid,
            description: "Data Not Valid".to_string(),
        }
    }

    pub fn new_as_precise() -> Self {
        Self {
            mode: "P".to_string(),
            mode_type: FAAModeType::Precise,
            description: "Precise (4.00 and later)".to_string(),
        }
    }

    pub fn new_as_rtk_integer() -> Self {
        Self {
            mode: "R".to_string(),
            mode_type: FAAModeType::RTKInteger,
            description: "RTK Integer mode".to_string(),
        }
    }

    pub fn new_as_simulated() -> Self {
        Self {
            mode: "S".to_string(),
            mode_type: FAAModeType::Simulated,
            description: "Simulated Mode".to_string(),
        }
    }

    pub fn new_as_unsafe() -> Self {
        Self {
            mode: "U".to_string(),
            mode_type: FAAModeType::Unsafe,
            description: "Quectel Querk, \"Unsafe\"".to_string(),
        }
    }

    pub fn new_as_unknown(value: &String) -> Self {
        Self {
            mode: value.to_owned(),
            mode_type: FAAModeType::Unsafe,
            description: format!("Unknown mode: {}", value).to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GgaQualityIndicator {
    FixNotAvailable = 0,
    GpsFix = 1,
    DifferentialGpsFix = 2,
    PPSFix = 3,
    RTKInteger = 4,
    RTKFloat = 5,
    Estimate = 6,
    ManualInputMode = 7,
    SimulationMode = 8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GpsDMS {
    pub degrees: u8,
    pub minutes: u8,
    pub seconds: u8,
}

impl GpsDMS {
    pub fn from_str(value: impl Into<&'static str>) -> Result<Self, NMEAError> {
        let dd_mm: &str = value.into();
        // Check the length is at least 6 DDMM.MM
        if dd_mm.len() < 7 || !dd_mm.contains(".") {
            return Err(NMEAError::InvalidData(
                format!("Couldn't convert value {} into a valid GPS DMS", &dd_mm).into(),
            ));
        }
        // DDMM.MM
        let degrees = dd_mm[..2].parse::<u8>()?;
        let minutes: u8 = dd_mm[2..4].parse::<u8>()?;
        let seconds: u8 = dd_mm[5..7].parse::<u8>()?;

        Ok(Self {
            degrees,
            minutes,
            seconds,
        })
    }
}

pub trait GpsDataFromNmeaString {
    type Output;
    /// Creates a GPS Data struct from a standard nmea string
    fn from_nmea_str(data: impl Into<String>) -> Result<Self::Output, NMEAError>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct GstData {
    /// UTC of position fix (GGA)
    pub fix_timestamp: Option<NaiveTime>,
    /// RMS value of the pseudorange residuals; includes carrier phase residuals during periods of RTK (float) and RTK (fixed) processing
    pub rms_dev: Option<f32>,
    /// Error ellipse semi-major axis 1-sigma error, in meters
    pub semi_major_dev: Option<f32>,
    /// Error ellipse semi-minor axis 1-sigma error, in meters
    pub semi_minor_dev: Option<f32>,
    /// Orientation of semi-major axis of error ellipse (true north degrees)
    pub semi_major_orientation: Option<f32>,
    /// Standard deviation (meters) of latitude error
    pub latitude_error: Option<f32>,
    /// Standard deviation (meters) of longitude error
    pub longitude_error: Option<f32>,
    /// Standard deviation (meters) of altitude error
    pub altitude_error: Option<f32>,
}

fn nmea_str_to_vec<'a, 'b: 'a>(data: impl Into<&'b String>) -> Vec<&'a str> {
    let items: Vec<&str> = data
        .into()
        .split(',')
        .map(|v| v.split('*').nth(0).unwrap_or(v)) // strip * from the end
        .collect();
    items
}

impl GpsDataFromNmeaString for GstData {
    type Output = Self;

    fn from_nmea_str(data: impl Into<String>) -> Result<Self::Output, NMEAError> {
        // All fields including the checksum
        const FIELD_COUNT: usize = 9;
        let data: String = data.into();
        let items = nmea_str_to_vec(&data);
        let result = match &items[0][3..] {
            "GST" => {
                if items.len() != FIELD_COUNT {
                    Err(NMEAError::InvalidData(
                        "GST sentence is not 9 fields in length".to_string(),
                    ))
                } else {
                    Ok(GstData {
                        fix_timestamp: NaiveTime::parse_from_str(&items[1], "%H%M%S.3f").ok(),
                        rms_dev: items[2].parse::<f32>().ok(),
                        semi_major_dev: items[3].parse::<f32>().ok(),
                        semi_minor_dev: items[4].parse::<f32>().ok(),
                        semi_major_orientation: items[5].parse::<f32>().ok(),
                        latitude_error: items[6].parse::<f32>().ok(),
                        longitude_error: items[7].parse::<f32>().ok(),
                        altitude_error: items[8].parse::<f32>().ok(),
                    })
                }
            }
            _ => Err(NMEAError::InvalidData(
                format!("GST raw value {} is invalid", &items[0][3..]).to_string(),
            )),
        }?;
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GsaSelectionMode {
    /// Manual mode, forced to operate in 2D or 3D
    Manual,
    /// Automatic, 2D/3D
    Automatic,
    /// Unknown selection mode
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GsaMode {
    /// 1 = no fix
    ModeNone,
    /// 2 = 2D fix
    Mode2D,
    /// 3 = 3D fix
    Mode3D,
    /// Unknown mode
    Unknown(String),
}

/// Note: NMEA 4.1+ systems (u-blox 9, Quectel LCD79) may emit an extra field, System ID, just before the checksum.
#[derive(Debug, Clone, PartialEq)]
pub enum SystemID {
    /// 1 = GPS L1C/A, L2CL, L2CM
    GPS,
    /// 2 = GLONASS L1 OF, L2 OF
    GLONASS,
    /// 3 = Galileo E1C, E1B, E5 bl, E5 bQ
    Galileo,
    /// 4 = BeiDou B1I D1, B1I D2, B2I D1, B2I D12
    BeiDou,
    /// Unknown System ID
    Unknown(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct GsaData {
    pub selection_mode: GsaSelectionMode,
    pub mode: GsaMode,
    pub prn_numbers: Vec<Option<u8>>,
    pub pdop: Option<f64>,
    pub hdop: Option<f64>,
    pub vdop: Option<f64>,
    ///  Signal ID (NMEA 4.11)
    pub system_id: Option<SystemID>,
}

impl GpsDataFromNmeaString for GsaData {
    type Output = Self;

    fn from_nmea_str(data: impl Into<String>) -> Result<Self::Output, NMEAError> {
        // All fields including the checksum
        const FIELD_COUNT: usize = 18;
        let data: String = data.into();
        let items = nmea_str_to_vec(&data);
        // Note: NMEA 4.1+ systems (u-blox 9, Quectel LCD79) may emit an extra field, System ID, just before the checksum.
        // Example: $GNGSA,A,3,80,71,73,79,69,,,,,,,,1.83,1.09,1.47*17
        let result = match &items[0][3..] {
            "GSA" => {
                if items.len() < FIELD_COUNT {
                    Err(NMEAError::InvalidData(
                        format!("GSA sentence is not {FIELD_COUNT} fields in length").to_string(),
                    ))
                } else {
                    let selection_mode = match items[1] {
                        "M" => GsaSelectionMode::Manual,
                        "A" => GsaSelectionMode::Automatic,
                        _ => GsaSelectionMode::Unknown(items[1].to_string()),
                    };
                    let mode = match items[2].parse::<u32>().ok() {
                        Some(0u32) => GsaMode::ModeNone,
                        Some(1u32) => GsaMode::Mode2D,
                        Some(2u32) => GsaMode::Mode3D,
                        _ => GsaMode::Unknown(items[2].to_string()),
                    };
                    Ok(GsaData {
                        selection_mode: selection_mode,
                        mode: mode,
                        prn_numbers: items[3..=14].iter().map(|v| v.parse::<u8>().ok()).collect(),
                        pdop: items[15].parse::<f64>().ok(),
                        hdop: items[16].parse::<f64>().ok(),
                        vdop: items[17].parse::<f64>().ok(),
                        system_id: {
                            if items.len() > FIELD_COUNT {
                                match items[18].parse::<u32>().ok() {
                                    Some(1u32) => Some(SystemID::GPS),
                                    Some(2u32) => Some(SystemID::GLONASS),
                                    Some(3u32) => Some(SystemID::Galileo),
                                    Some(4u32) => Some(SystemID::BeiDou),
                                    _ => Some(SystemID::Unknown(items[18].to_string())),
                                }
                            } else {
                                None
                            }
                        }
                    })
                }
            }
            _ => Err(NMEAError::InvalidData(
                format!("GSA raw value {} is invalid", &items[0][3..]).to_string(),
            )),
        }?;
        Ok(result)
    }
}

/// These sentences describe the sky position of a UPS satellite in view. Typically they’re shipped in a group of 2 or 3.
/// 
/// Note: Some GPS receivers may emit more than 12 quadruples (more than three GPGSV sentences), even though 
/// NMEA-0813 doesn’t allow this. (The extras might be WAAS satellites, for example.) Receivers may also
/// report quads for satellites they aren’t tracking, in which case the SNR field will be null; we don’t
/// know whether this is formally allowed or not.
/// 
/// Note: NMEA 4.10+ systems (u-blox 9, Quectel LCD79) may emit an extra field, Signal ID, just before the 
/// checksum. See the description of Signal ID’s above.
/// 
/// Note: $GNGSV uses PRN in field 4. Other $GxGSV use the satellite ID in field 4. Jackson Labs, Quectel, 
/// Telit, and others get this wrong, in various conflicting ways.
/// 
#[derive(Clone, Debug, PartialEq)]
pub struct GsvData {
    /// total number of GSV sentences to be transmitted in this group
    pub total_count: Option<u16>,
    /// Sentence number, 1-9 of this GSV message within current group
    pub count: Option<u16>,
    /// total number of satellites in view (leading zeros sent)
    pub sat_in_view: Option<u16>,
    /// satellite ID or PRN number (leading zeros sent)
    pub id_or_prn_number: Option<u16>,
    /// elevation in degrees (-90 to 90) (leading zeros sent)
    pub elevation: Option<i16>,
    /// azimuth in degrees to true north (000 to 359) (leading zeros sent)
    pub azimuth: Option<u16>,
    /// SNR in dB (00-99) (leading zeros sent) more satellite info quadruples
    pub snr: Option<u16>,
    ///  Signal ID (NMEA 4.11)
    pub system_id: Option<SystemID>,
}

impl GpsDataFromNmeaString for GsvData {
    type Output = Self;

    fn from_nmea_str(data: impl Into<String>) -> Result<Self::Output, NMEAError> {
        // All fields including the checksum
        const FIELD_COUNT: usize = 8;
        let data: String = data.into();
        // Example: $GPGSV,3,1,11,03,03,111,00,04,15,270,00,06,01,010,00,13,06,292,00*74
        let items = nmea_str_to_vec(&data);
        // Note: NMEA 4.1+ systems (u-blox 9, Quectel LCD79) may emit an extra field, System ID, just before the checksum.
        let result = match &items[0][3..] {
            "GSV" => {
                if items.len() < FIELD_COUNT {
                    Err(NMEAError::InvalidData(
                        format!("GSV sentence is not {FIELD_COUNT} fields in length").to_string(),
                    ))
                } else {
                    Ok(GsvData {
                    total_count: items[1].parse::<u16>().ok(),
                    count: items[2].parse::<u16>().ok(),
                    sat_in_view: items[3].parse::<u16>().ok(),
                    id_or_prn_number: items[4].parse::<u16>().ok(),
                    elevation: items[5].parse::<i16>().ok(),
                    azimuth: items[6].parse::<u16>().ok(),
                    snr: items[7].parse::<u16>().ok(),
                    system_id: {
                        if items.len() > FIELD_COUNT {
                            match items[8].parse::<u32>().ok() {
                                Some(1u32) => Some(SystemID::GPS),
                                Some(2u32) => Some(SystemID::GLONASS),
                                Some(3u32) => Some(SystemID::Galileo),
                                Some(4u32) => Some(SystemID::BeiDou),
                                _ => Some(SystemID::Unknown(items[18].to_string())),
                            }
                        } else {
                            None
                        }
                    }})
                }
            }
            _ => Err(NMEAError::InvalidData(
                format!("GSA raw value {} is invalid", &items[0][3..]).to_string(),
            )),
        }?;
        Ok(result)
    }
}

#[repr(transparent)]
#[derive(Clone, Debug, PartialEq)]
pub struct GsvDataCollection { inner: Vec<GsvData> }

impl GpsDataFromNmeaString for GsvDataCollection {
    type Output = Self;

    fn from_nmea_str(data: impl Into<String>) -> Result<Self::Output, NMEAError> {
        // All fields including the checksum
        const FIELD_COUNT: usize = 8;
        let data: String = data.into();
        let mut gsv_items = Vec::new();
        // GSV messages have a lot of individual $GxGSV messages seperated by a space
        // Example: $GPGSV,3,1,11,03,03,111,00,04,15,270,00,06,01,010,00,13,06,292,00*74 $GPGSV,3,2,11,14,25,170,00,16,57,208,39,18,67,296,40,19,40,246,00*74 $GPGSV,3,3,11,22,42,067,42,24,14,311,43,27,05,244,00,,,,*4D
        for gsv_data in data.split(" ").collect::<Vec<&str>>() {
            let gsv = GsvData::from_nmea_str(gsv_data)?;
            gsv_items.push(gsv);
        }
        Ok(GsvDataCollection { inner: gsv_items })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GllData {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timestamp: Option<NaiveTime>,
    pub data_valid: Option<bool>,
    pub faa_mode: Option<FAAMode>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GgaData {
    pub timestamp: Option<NaiveTime>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub quality: GgaQualityIndicator,
    pub satellite_count: Option<u8>,
    pub hdop: Option<f64>,
    pub altitude: Option<f64>,
    pub geoid_separation: Option<f64>,
    pub age_of_dgps: Option<f64>,
    pub ref_station_id: Option<u16>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VtgData {
    pub cog_true: Option<f64>,
    pub cog_magnetic: Option<f64>,
    pub sog_knots: Option<f64>,
    pub sog_kph: Option<f64>,
    pub faa_mode: Option<FAAMode>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RmcData {
    pub timestamp: Option<NaiveTime>,
    pub status_active: Option<bool>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub sog_knots: Option<f64>,
    pub bearing: Option<f64>,
    pub variation: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Pubx00Data {
    raw: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Pubx03Data {
    raw: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Pubx04Data {
    raw: String,
}

/// Most GPS sensors emit only RMC, GSA, GSV, GLL, VTG, and (rarely) ZDA.
/// Newer ones conforming to NMEA 3.x may emit GBS as well. Other NMEA sentences
/// are usually only emitted by high-end maritime navigation systems.
/// In NMEA 2.3, several sentences (APB, BWC, BWR, GLL, RMA, RMB, RMC, VTG, WCV, and XTE)
/// got a new last field carrying the signal integrity information needed by the FAA.
/// (The values in the GGA mode field were extended to carry this information as well.)
#[derive(Clone, Debug, PartialEq)]
pub enum NMEASentenceType {
    /// GPS Pseudorange Noise Statistics
    GST(GstData),
    /// GPS DOP and active satellites
    GSA(GsaData),
    /// Satellites in view
    GSV(GsvDataCollection),
    /// Geographic Position - Latitude/Longitude
    GLL(GllData),
    /// Global Positioning System Fix Data
    GGA(GgaData),
    /// Track made good and Ground speed
    VTG(VtgData),
    /// Recommended Minimum Navigation Information
    RMC(RmcData),
    /// u-blox Lat/Long Position Data
    PUBX00(Pubx00Data),
    /// u-blox Satellite Status
    PUBX03(Pubx03Data),
    /// u-blox Time of Day and Clock Information
    PUBX04(Pubx04Data),
    /// Unknown
    Unsupported(String),
}

impl fmt::Display for NMEASentenceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::GST(_) => write!(f, "GST"),
            Self::GSA(_) => write!(f, "GSA"),
            Self::GSV(_) => write!(f, "GSV"),
            Self::GLL(_) => write!(f, "GLL"),
            Self::GGA(_) => write!(f, "GGA"),
            Self::VTG(_) => write!(f, "VTG"),
            Self::RMC(_) => write!(f, "RMC"),
            Self::PUBX00(_) => write!(f, "PUBX00"),
            Self::PUBX03(_) => write!(f, "PUBX03"),
            Self::PUBX04(_) => write!(f, "PUBX04"),
            Self::Unsupported(s) => write!(f, "{}", s),
        }
    }
}
