use crate::types::{Error, Result};
use core::time;

use regex::Regex;
use sfml::{self, audio::{SoundBufferRecorder, Sound, SoundBuffer}};

#[derive(Debug)]
pub struct Audio {
    /// Typically is "Monitor of PCM2912A Audio Codec Analog Stereo"
    pub capture_name: String,
    /// Index of the capture device if multiple, starts at 1
    /// "Monitor of PCM2912A Audio Codec Analog Stereo #2" would be an index of 2
    pub index: u32,
    recorder: SoundBufferRecorder,
}

impl Audio {
    pub fn find_neovi_mic2_audio() -> Result<Vec<Self>> {
        // "PCM2912A Audio Codec Analog Stereo"
        // "PCM2912A Audio Codec Analog Stereo #2"
        let re = Regex::new(r"^PCM2912A Audio Codec").unwrap();
        let re_index = Regex::new(r"\d+$").unwrap();
        let mut capture_devices = Vec::new();

        if !sfml::audio::capture::is_available() {
            return Err(Error::CriticalError("Audio capture is unavailable!".into()));
        }
        let devices = sfml::audio::capture::available_devices();
        for device in &*devices {
            //println!("{}", device.to_str().unwrap());
            // Match our expected audio device
            if !re.is_match(device.to_str().unwrap()) {
                continue;
            }
            // Find the index of the capture device
            let index = match re_index.find(device.to_str().unwrap()) {
                Some(m) => m.as_str().parse::<u32>().unwrap(),
                None => 1,
            };
            // Create the recorder
            let mut recorder = SoundBufferRecorder::new();
            let name = device.to_str().unwrap();
            recorder.set_device(name).unwrap();
            // Create the Audio device
            capture_devices.push(Self {
                capture_name: device.to_string(),
                index,
                recorder,
            });
        }
        Ok(capture_devices)
    }

    pub fn start(&mut self, sample_rate: u32) -> Result<()> {
        if !self.recorder.start(sample_rate) {
            return Err(Error::CriticalError("Failed to start recording!".into()))
        }
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.recorder.stop();
        Ok(())
    }

    pub fn save_to_file(&mut self, fname: impl Into<String>) -> Result<()> {
        let fname: String = fname.into();
        if !self.recorder.buffer().save_to_file(fname.as_str()) {
            return Err(Error::CriticalError(
                format!("Failed to save capture from {} to file {}", self.capture_name, fname).into())
            );
        }
        Ok(())
    }
}

pub fn record_default_capture() {
    if !sfml::audio::capture::is_available() {
        panic!("Audio capture device is unavailable!");
    }

    let mut recorder = sfml::audio::SoundBufferRecorder::default();
    println!("Recording {}...", recorder.device());
    if !recorder.start(44100) {
        panic!("Failed to start recording!");
    }
    std::thread::sleep(time::Duration::from_millis(3000));
    recorder.stop();
    println!("Finished Recording");

    let buffer = recorder.buffer();
    if !buffer.save_to_file("/home/drebbe/test.wav") {
        panic!("Failed to save file!");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_find_neovi_mic2_capture() -> Result<()> {
        let mut devices = Audio::find_neovi_mic2_audio()?;
        println!("{devices:#?}");
        for (i, device) in devices.iter_mut().enumerate() {
            println!("Recording {}", device.capture_name);
            device.start(44_100)?;
            std::thread::sleep(std::time::Duration::from_secs_f64(3.0));
            device.stop()?;
            device.save_to_file(format!("save_dev{i}.ogg").to_string())?;
        }
        Ok(())
    }
    #[test]
    fn test_record_default_capture() {
        record_default_capture();
    }
}
