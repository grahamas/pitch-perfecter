use sound_synth::voice_synth::voice_like_single_pitch;
use hound::{WavWriter, WavSpec, SampleFormat};

fn main() {
    let sample_rate = 44100.0;
    let duration_sec = 5.0;
    let len = (sample_rate * duration_sec) as usize;
    let freq = 220.0;
    let vibrato_freq = 5.0;
    let vibrato_depth = 8.0;
    let harmonics = 8;
    let signal = voice_like_single_pitch(freq, harmonics, sample_rate, len);

    let spec = WavSpec {
        channels: 1,
        sample_rate: sample_rate as u32,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };
    let mut writer = WavWriter::create("voice_like_test.wav", spec).unwrap();
    for s in signal {
        writer.write_sample(s).unwrap();
    }
    writer.finalize().unwrap();
    println!("Wrote voice_like_test.wav");
}
