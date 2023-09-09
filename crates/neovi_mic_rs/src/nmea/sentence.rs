use super::types::{GstData, GsaData, GsvDataCollection, GllData, GgaData, VtgData, RmcData, Pubx00Data, Pubx03Data, Pubx04Data, NMEAError, NMEASentenceType, GpsDataFromNmeaString};
use chrono::NaiveTime;

#[derive(Debug, Clone)]
pub struct NMEASentence {
    pub raw_data: String,
}

impl NMEASentence {
    pub fn new(raw_data: impl Into<String>) -> Result<Self, NMEAError> {
        Ok(Self {
            raw_data: raw_data.into(),
        })
    }

    pub fn data(&self) -> Result<NMEASentenceType, NMEAError> {
        // Split the raw data into a vec
        let items: Vec<&str> = self
            .raw_data
            .split(',')
            .map(|v| v.split('*').nth(0).unwrap_or(v)) // strip * from the end
            .collect();
        let result = match &items[0][3..] {
            "GST" => Ok(NMEASentenceType::GST(GstData::from_nmea_str(&self.raw_data)?)),
            "GSA" => Ok(NMEASentenceType::GSA(GsaData::from_nmea_str(&self.raw_data)?)),
            "GSV" => Ok(NMEASentenceType::GSV(GsvDataCollection::from_nmea_str(&self.raw_data)?)),
            // "GLL" => Ok(NMEASentenceType::GLL(GllData::from_nmea_str(&self.raw_data)?)),
            // "GGA" => Ok(NMEASentenceType::GGA(GgaData::from_nmea_str(&self.raw_data)?)),
            // "VTG" => Ok(NMEASentenceType::VTG(VtgData::from_nmea_str(&self.raw_data)?)),
            // "RMC" => Ok(NMEASentenceType::RMC(RmcData::from_nmea_str(&self.raw_data)?)),
            // "PUBX00" => Ok(NMEASentenceType::PUBX00(Pubx00Data::from_nmea_str(&self.raw_data)?)),
            // "PUBX03" => Ok(NMEASentenceType::PUBX03(Pubx03Data::from_nmea_str(&self.raw_data)?)),
            // "PUBX04" => Ok(NMEASentenceType::PUBX04(Pubx04Data::from_nmea_str(&self.raw_data)?)),
            _ => Err(NMEAError::InvalidData(self.raw_data.to_owned())),
        };
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nmea_sentence_gst() {
        let sentence =
            NMEASentence::new("$GPGST,182141.000,15.5,15.3,7.2,21.8,0.9,0.5,0.8*54").unwrap();
        let data = sentence.data().unwrap();
        println!("{data:#?}");
    }

    #[test]
    fn test_nmea_sentence_gsa() {
        let sentence =
            NMEASentence::new("$GNGSA,A,3,80,71,73,79,69,,,,,,,,1.83,1.09,1.47*17").unwrap();
        let data = sentence.data().unwrap();
        println!("{data:#?}");
    }

    #[test]
    fn test_nmea_sentence_gsv() {
        let sentence =
            NMEASentence::new("$GPGSV,3,1,11,03,03,111,00,04,15,270,00,06,01,010,00,13,06,292,00*74 $GPGSV,3,2,11,14,25,170,00,16,57,208,39,18,67,296,40,19,40,246,00*74 $GPGSV,3,3,11,22,42,067,42,24,14,311,43,27,05,244,00,,,,*4D").unwrap();
        let data = sentence.data().unwrap();
        println!("{data:#?}");
    }
}
