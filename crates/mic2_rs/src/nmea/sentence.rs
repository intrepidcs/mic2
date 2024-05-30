use super::types::{
    nmea_str_to_vec, GpsDataFromNmeaString, GsaData, GstData, GsvDataCollection, NMEAError,
    NMEASentenceType, Pubx00Data, Pubx03Data, Pubx04Data,
};

/// Represents a GPS NMEA Sentence
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct NMEASentence {
    pub inner: String,
}

impl NMEASentence {
    /// Creates a new [NMEASentence] from a string
    ///
    /// Example:
    /// ```
    /// use mic2::nmea::sentence::NMEASentence;
    /// use mic2::nmea::types::NMEASentenceType;
    ///
    /// let sentence =
    /// NMEASentence::new("$GPGST,182141.000,15.5,15.3,7.2,21.8,0.9,0.5,0.8*54").unwrap();
    /// let data = sentence.data().unwrap();
    /// println!("{data:#?}");
    /// match data {
    ///     NMEASentenceType::GST(d) => assert_eq!(d.semi_major_orientation, Some(21.8)),
    ///     _ => panic!("NMEA sentence wasn't GST..."),
    /// }
    /// ```
    pub fn new(raw_data: impl Into<String>) -> Result<Self, NMEAError> {
        Ok(Self {
            inner: raw_data.into().replace("\r\n", ""),
        })
    }

    /// Creates a new Vec<Result<[NMEASentence], [NMEAError]>> from a byte array.
    /// Expects the byte array to be able to convert into a UTF-8 String.
    pub fn from_bytes(bytes: &[u8]) -> Vec<Result<Self, NMEAError>> {
        // Convert to string
        let mut sentences = Vec::new();
        let inner = match String::from_utf8(bytes.to_vec()) {
            Ok(s) => s,
            Err(_) => {
                sentences.push(Err(NMEAError::InvalidData(
                    "Failed to Create NMEA sentence from bytes".into(),
                )));
                return sentences;
            }
        };

        for sentence in inner.split("\r\n") {
            let is_start = Self::is_start(sentence);
            let has_checksum = Self::contains_checksum(sentence);
            sentences.push(match (is_start, has_checksum) {
                (true, false) => Err(NMEAError::PartialStart(sentence.into())),
                (false, false) => Err(NMEAError::Partial(sentence.into())),
                (false, true) => Err(NMEAError::PartialEnd(sentence.into())),
                (true, true) => Ok(Self {
                    inner: sentence.into(),
                }),
            });
        }
        sentences
    }

    /// Checks the NMEA sentence to see if it contains a $ at the beginning of the sentence.
    /// Returns true if it contains a $ at the start, false otherwise.
    pub fn is_start<'a>(sentence: impl Into<&'a str>) -> bool {
        // match checksum pattern at the end of a nmea sentence. * followed by two hexidecimal digits
        regex::Regex::new(r"^\$").unwrap().is_match(sentence.into())
    }

    /// Checks the NMEA sentence to see if it contains a checksum at the end.
    /// Returns true if it contains a checksum, false otherwise.
    pub fn contains_checksum<'a>(sentence: impl Into<&'a str>) -> bool {
        // match checksum pattern at the end of a nmea sentence. * followed by two hexidecimal digits
        regex::Regex::new(r"\*[0-9a-fA-F]{2}")
            .unwrap()
            .is_match(sentence.into())
    }

    /// Returns the NMEASentenceType for parsing, NMEAError on error.
    pub fn data(&self) -> Result<NMEASentenceType, NMEAError> {
        // Split the raw data into a vec
        let items = nmea_str_to_vec(&self.inner);
        match &items[0][0..] {
            "$GNGST" | "$GPGST" => Ok(NMEASentenceType::GST(GstData::from_nmea_str(&self.inner)?)),
            "$GNGSA" | "$GPGSA" => Ok(NMEASentenceType::GSA(GsaData::from_nmea_str(&self.inner)?)),
            "$GNGSV" | "$GPGSV" => Ok(NMEASentenceType::GSV(GsvDataCollection::from_nmea_str(
                &self.inner,
            )?)),
            // "GLL" => Ok(NMEASentenceType::GLL(GllData::from_nmea_str(&self.inner)?)),
            // "GGA" => Ok(NMEASentenceType::GGA(GgaData::from_nmea_str(&self.inner)?)),
            // "VTG" => Ok(NMEASentenceType::VTG(VtgData::from_nmea_str(&self.inner)?)),
            // "RMC" => Ok(NMEASentenceType::RMC(RmcData::from_nmea_str(&self.inner)?)),
            // "GNTXT" => Ok(NMEASentenceType::GNTXT(GNTXTData::from_nmea_str(&self.inner)?)),
            "$PUBX" => match items[1] {
                "00" => Ok(NMEASentenceType::PUBX00(Pubx00Data::from_nmea_str(
                    &self.inner,
                )?)),
                "03" => Ok(NMEASentenceType::PUBX03(Pubx03Data::from_nmea_str(
                    &self.inner,
                )?)),
                "04" => Ok(NMEASentenceType::PUBX04(Pubx04Data::from_nmea_str(
                    &self.inner,
                )?)),
                _ => Err(NMEAError::InvalidData(self.inner.to_owned())),
            },
            _ => Err(NMEAError::InvalidData(self.inner.to_owned())),
        }
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

    #[test]
    fn test_pubx00_sentence() {
        let sentence =
            NMEASentence::new("$PUBX,00,025554.00,0000.00000,N,00000.00000,E,0.000,NF,5311696,3755936,0.000,0.00,0.000,,99.99,99.99,99.99,0,0,0*28\r\n").unwrap();
        let data = sentence.data().unwrap();
        println!("{data:#?}");
    }

    #[test]
    fn test_pubx03_empty_sentence() {
        let sentence = NMEASentence::new("$PUBX,03,00*1C\r\n").unwrap();
        let data = sentence.data().unwrap();
        println!("{data:#?}");
    }

    #[test]
    fn test_pubx03_sentence() {
        let sentence =
            NMEASentence::new("$PUBX,03,06,2,U,137,37,24,000,8,U,053,52,28,064,9,U,202,12,21,000,14,-,,,22,000,27,-,049,16,,000,81,-,,,08,000*54\r\n").unwrap();
        let data = sentence.data().unwrap();
        println!("{data:#?}");
    }
}
