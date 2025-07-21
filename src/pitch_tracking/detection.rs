use crate::audio::Audio;
use pitch_detection;

trait PitchDetector {
    fn get_pitch<T: Audio>(&mut self, audio: T) -> Option<pitch_detection::Pitch>;
}

trait MonoPitchDetector: PitchDetector {
    fn get_mono_pitch<T: MonoAudioSource>(&mut self, mono_audio: T) -> Option<pitch_detection::Pitch>;
    fn get_pitch<T: MonoAudioSource>(&mut self, audio: T) -> Option<pitch_detection::Pitch> {
        self.get_mono_pitch(audio)
    }
}