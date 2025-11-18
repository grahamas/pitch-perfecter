use sound_synth::voice_like_single_pitch;
use audio_utils::{MonoAudio, io::save_wav};

fn main() {
    let sample_rate = 44100.0;
    let duration_sec = 5.0;
    let len = (sample_rate * duration_sec) as usize;
    let freq = 220.0;
    let _vibrato_freq = 5.0;
    let _vibrato_depth = 8.0;
    let harmonics = 8;
    let signal = voice_like_single_pitch(freq, harmonics, sample_rate, len);

    let audio = MonoAudio::new(signal, sample_rate as u32);
    save_wav("voice_like_test.wav", &audio).expect("Failed to save audio file");
    println!("Wrote voice_like_test.wav");
}
