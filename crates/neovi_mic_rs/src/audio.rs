use core::time;

use sfml;

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
    if !buffer.save_to_file("~/test.wav") {
        panic!("Failed to save file!");
    }
}

