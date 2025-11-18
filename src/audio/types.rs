//! Audio Types and Traits
//!
//! This module defines the core types and traits for audio processing.

/// Base trait for all audio types
/// 
/// This trait provides the fundamental interface that all audio types must implement.
/// It ensures that any audio type can report its sample rate, which is essential
/// for frequency-based operations like pitch detection.
pub trait Audio {
    /// Returns the sample rate in Hz
    fn sample_rate(&self) -> u32;
}

/// Trait for audio sources that can provide mono (single-channel) audio data
/// 
/// This trait is implemented by audio types that can provide access to mono audio samples.
/// For stereo or multi-channel audio, implementations might mix down to mono or extract
/// a specific channel.
pub trait MonoAudioSource: Audio {
    /// Returns a slice of mono audio samples
    /// 
    /// The samples are represented as f32 values, typically in the range [-1.0, 1.0].
    fn mono_samples(&self) -> &[f32];
}

/// Trait for audio types that support windowed iteration
/// 
/// This trait enables processing audio in overlapping or non-overlapping windows,
/// which is essential for time-frequency analysis and pitch tracking.
pub trait IterableAudio: Audio {
    /// Returns an iterator over sliding windows of audio samples
    /// 
    /// # Arguments
    /// * `window_size` - The number of samples in each window
    /// * `step_size` - The number of samples to advance between windows (hop size)
    /// 
    /// # Returns
    /// An iterator that yields windows of audio data. Each window is itself
    /// an audio object that implements the same audio traits.
    fn sliding_windows(&self, window_size: usize, step_size: usize) -> SlidingWindows<'_>;
}

/// MonoAudio represents a single-channel audio buffer
/// 
/// This is the primary concrete type for working with mono audio data.
/// It stores audio samples in memory and provides all the necessary
/// interfaces for audio processing operations.
/// 
/// # Fields
/// * `samples` - The audio sample data as 32-bit floating point values
/// * `sample_rate` - The sample rate in Hz (samples per second)
#[derive(Debug, Clone)]
pub struct MonoAudio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

impl MonoAudio {
    /// Create a new MonoAudio instance
    /// 
    /// # Arguments
    /// * `samples` - Vector of audio samples
    /// * `sample_rate` - Sample rate in Hz
    /// 
    /// # Returns
    /// A new MonoAudio instance
    pub fn new(samples: Vec<f32>, sample_rate: u32) -> Self {
        MonoAudio {
            samples,
            sample_rate,
        }
    }
}

impl Audio for MonoAudio {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

impl MonoAudioSource for MonoAudio {
    fn mono_samples(&self) -> &[f32] {
        &self.samples
    }
}

impl IterableAudio for MonoAudio {
    fn sliding_windows(&self, window_size: usize, step_size: usize) -> SlidingWindows<'_> {
        SlidingWindows {
            samples: &self.samples,
            sample_rate: self.sample_rate,
            window_size,
            step_size,
            position: 0,
        }
    }
}

/// Iterator for sliding windows over audio samples
/// 
/// This iterator yields MonoAudio instances, each representing a window
/// of audio samples from the original buffer.
pub struct SlidingWindows<'a> {
    samples: &'a [f32],
    sample_rate: u32,
    window_size: usize,
    step_size: usize,
    position: usize,
}

impl<'a> Iterator for SlidingWindows<'a> {
    type Item = MonoAudio;
    
    fn next(&mut self) -> Option<Self::Item> {
        // Check if we have enough samples left for a full window
        if self.position + self.window_size > self.samples.len() {
            return None;
        }
        
        // Extract the window
        let window = &self.samples[self.position..self.position + self.window_size];
        let window_audio = MonoAudio {
            samples: window.to_vec(),
            sample_rate: self.sample_rate,
        };
        
        // Advance position
        self.position += self.step_size;
        
        Some(window_audio)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mono_audio_creation() {
        let samples = vec![0.0, 0.5, 1.0, 0.5, 0.0];
        let audio = MonoAudio::new(samples.clone(), 44100);
        
        assert_eq!(audio.samples, samples);
        assert_eq!(audio.sample_rate, 44100);
    }

    #[test]
    fn test_mono_audio_source_trait() {
        let samples = vec![0.1, 0.2, 0.3];
        let audio = MonoAudio::new(samples.clone(), 48000);
        
        assert_eq!(audio.sample_rate(), 48000);
        assert_eq!(audio.mono_samples(), &samples[..]);
    }

    #[test]
    fn test_audio_trait() {
        let audio = MonoAudio::new(vec![1.0], 22050);
        let audio_ref: &dyn Audio = &audio;
        
        assert_eq!(audio_ref.sample_rate(), 22050);
    }

    #[test]
    fn test_sliding_windows_basic() {
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let audio = MonoAudio::new(samples, 44100);
        
        let windows: Vec<_> = audio.sliding_windows(2, 2).collect();
        
        assert_eq!(windows.len(), 3);
        assert_eq!(windows[0].samples, vec![1.0, 2.0]);
        assert_eq!(windows[1].samples, vec![3.0, 4.0]);
        assert_eq!(windows[2].samples, vec![5.0, 6.0]);
    }

    #[test]
    fn test_sliding_windows_overlap() {
        let samples = vec![1.0, 2.0, 3.0, 4.0];
        let audio = MonoAudio::new(samples, 44100);
        
        let windows: Vec<_> = audio.sliding_windows(2, 1).collect();
        
        assert_eq!(windows.len(), 3);
        assert_eq!(windows[0].samples, vec![1.0, 2.0]);
        assert_eq!(windows[1].samples, vec![2.0, 3.0]);
        assert_eq!(windows[2].samples, vec![3.0, 4.0]);
    }

    #[test]
    fn test_sliding_windows_incomplete() {
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let audio = MonoAudio::new(samples, 44100);
        
        // Window size 3, step 2: [1,2,3], [3,4,5]
        let windows: Vec<_> = audio.sliding_windows(3, 2).collect();
        
        assert_eq!(windows.len(), 2);
        assert_eq!(windows[0].samples, vec![1.0, 2.0, 3.0]);
        assert_eq!(windows[1].samples, vec![3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_sliding_windows_preserve_sample_rate() {
        let audio = MonoAudio::new(vec![1.0, 2.0, 3.0, 4.0], 48000);
        let windows: Vec<_> = audio.sliding_windows(2, 2).collect();
        
        for window in windows {
            assert_eq!(window.sample_rate(), 48000);
        }
    }

    #[test]
    fn test_mono_audio_clone() {
        let audio = MonoAudio::new(vec![1.0, 2.0, 3.0], 44100);
        let cloned = audio.clone();
        
        assert_eq!(audio.samples, cloned.samples);
        assert_eq!(audio.sample_rate, cloned.sample_rate);
    }
}
