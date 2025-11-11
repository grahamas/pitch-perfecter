//! Audio module providing audio types and traits for pitch detection
//! 
//! This module defines core audio types and traits used throughout the pitch detection system.

/// Trait for audio sources that provide mono audio samples
pub trait MonoAudioSource {
    /// Get the sample rate of the audio
    fn sample_rate(&self) -> f32;
    
    /// Get the mono audio samples
    fn mono_samples(&self) -> Vec<f32>;
}

/// Trait for audio types that can produce an audio object
pub trait Audio {
    type AudioType: MonoAudioSource;
    fn audio(&self) -> Self::AudioType;
}

/// Trait for audio that can be iterated with sliding windows
pub trait IterableAudio: MonoAudioSource {
    /// Create sliding windows over the audio samples
    fn sliding_windows(&self, window_size: usize, step_size: usize) -> SlidingWindowIterator;
}

/// A simple mono audio structure with samples and sample rate
#[derive(Clone, Debug)]
pub struct MonoAudio {
    pub samples: Vec<f32>,
    pub sample_rate: f32,
}

impl MonoAudio {
    /// Create a new MonoAudio instance
    pub fn new(samples: Vec<f32>, sample_rate: impl Into<f64>) -> Self {
        Self {
            samples,
            sample_rate: sample_rate.into() as f32,
        }
    }
}

impl MonoAudioSource for MonoAudio {
    fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
    
    fn mono_samples(&self) -> Vec<f32> {
        self.samples.clone()
    }
}

impl IterableAudio for MonoAudio {
    fn sliding_windows(&self, window_size: usize, step_size: usize) -> SlidingWindowIterator {
        SlidingWindowIterator::new(self.samples.clone(), self.sample_rate, window_size, step_size)
    }
}

/// Iterator for sliding windows over audio samples
pub struct SlidingWindowIterator {
    samples: Vec<f32>,
    sample_rate: f32,
    window_size: usize,
    step_size: usize,
    position: usize,
}

impl SlidingWindowIterator {
    fn new(samples: Vec<f32>, sample_rate: f32, window_size: usize, step_size: usize) -> Self {
        Self {
            samples,
            sample_rate,
            window_size,
            step_size,
            position: 0,
        }
    }
}

impl Iterator for SlidingWindowIterator {
    type Item = MonoAudio;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.position + self.window_size > self.samples.len() {
            return None;
        }
        
        let window_samples = self.samples[self.position..self.position + self.window_size].to_vec();
        self.position += self.step_size;
        
        Some(MonoAudio {
            samples: window_samples,
            sample_rate: self.sample_rate,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mono_audio_source() {
        let audio = MonoAudio {
            samples: vec![1.0, 2.0, 3.0],
            sample_rate: 44100.0,
        };
        assert_eq!(audio.sample_rate(), 44100.0);
        assert_eq!(audio.mono_samples(), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_sliding_windows() {
        let audio = MonoAudio {
            samples: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            sample_rate: 44100.0,
        };
        
        let windows: Vec<_> = audio.sliding_windows(2, 2).collect();
        assert_eq!(windows.len(), 3);
        assert_eq!(windows[0].samples, vec![1.0, 2.0]);
        assert_eq!(windows[1].samples, vec![3.0, 4.0]);
        assert_eq!(windows[2].samples, vec![5.0, 6.0]);
    }

    #[test]
    fn test_sliding_windows_overlap() {
        let audio = MonoAudio {
            samples: vec![1.0, 2.0, 3.0, 4.0],
            sample_rate: 44100.0,
        };
        
        let windows: Vec<_> = audio.sliding_windows(2, 1).collect();
        assert_eq!(windows.len(), 3);
        assert_eq!(windows[0].samples, vec![1.0, 2.0]);
        assert_eq!(windows[1].samples, vec![2.0, 3.0]);
        assert_eq!(windows[2].samples, vec![3.0, 4.0]);
    }
}
