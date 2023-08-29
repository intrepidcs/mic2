use super::types::{GstData, NMEAError, NMEASentenceType};
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
            "GST" => {
                if items.len() != 9 {
                    Err(NMEAError::InvalidData(
                        "GST sentence is not 9 fields in length".to_string(),
                    ))
                } else {
                    Ok(NMEASentenceType::GST(GstData {
                        fix_timestamp: NaiveTime::parse_from_str(&items[1], "%H%M%S.3f").ok(),
                        rms_dev: items[2].parse::<f32>().ok(),
                        semi_major_dev: items[3].parse::<f32>().ok(),
                        semi_minor_dev: items[4].parse::<f32>().ok(),
                        semi_major_orientation: items[5].parse::<f32>().ok(),
                        latitude_error: items[6].parse::<f32>().ok(),
                        longitude_error: items[7].parse::<f32>().ok(),
                        altitude_error: items[8].parse::<f32>().ok(),
                    }))
                }
            }
            _ => Err(NMEAError::InvalidData(
                "GST raw value is invalid".to_string(),
            )),
        }?;
        Ok(result)
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
        println!("{data:?}");
    }
}
