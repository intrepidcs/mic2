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

#[derive(Clone, Debug, PartialEq)]
pub struct GsaData {
    pub mode1_automatic: Option<bool>,
    // TODO: pub mode2_3d
    pub prn_numbers: Vec<u8>,
    pub pdop: Option<f64>,
    pub hdop: Option<f64>,
    pub vdop: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GsvData {
    pub prn_number: u8,
    pub elevation: Option<f32>,
    pub azimuth: Option<f32>,
    pub snr: Option<f32>,
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
    GSV(GsvData),
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
