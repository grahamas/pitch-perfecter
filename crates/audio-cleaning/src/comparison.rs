//! Filtering comparison module
//!
//! This module provides functionality to compare audio before and after filtering,
//! enabling review of the effectiveness of different filtering methods.
//!
//! Features:
//! - Store before/after audio for comparison
//! - Save audio pairs to disk for review
//! - Compute and compare spectra
//! - Export comparison data for visualization

use audio_utils::{MonoAudio, io::{save_wav, AudioIoError}};
use super::types::Spectrum;
use std::path::Path;

/// Holds audio and spectral data for before/after filtering comparison
#[derive(Clone)]
pub struct FilteringComparison {
    /// Original audio before filtering
    pub before: MonoAudio,
    /// Audio after filtering
    pub after: MonoAudio,
    /// Spectrum of the original audio (computed lazily)
    pub before_spectrum: Option<Spectrum>,
    /// Spectrum of the filtered audio (computed lazily)
    pub after_spectrum: Option<Spectrum>,
}

impl FilteringComparison {
    /// Create a new filtering comparison from before and after audio
    ///
    /// # Arguments
    /// * `before` - Original audio before filtering
    /// * `after` - Audio after filtering
    ///
    /// # Returns
    /// A new FilteringComparison instance
    ///
    /// # Example
    /// ```
    /// use audio_utils::MonoAudio;
    /// use audio_cleaning::{FilteringComparison, clean_audio_for_pitch};
    ///
    /// let original = MonoAudio::new(vec![0.0, 1.0, 0.0, -1.0], 44100);
    /// let filtered = clean_audio_for_pitch(&original, None, None);
    /// let comparison = FilteringComparison::new(original, filtered);
    /// ```
    pub fn new(before: MonoAudio, after: MonoAudio) -> Self {
        Self {
            before,
            after,
            before_spectrum: None,
            after_spectrum: None,
        }
    }

    /// Compute and store the spectra for both before and after audio
    ///
    /// This method computes the FFT spectra of the audio signals for frequency domain analysis.
    ///
    /// # Example
    /// ```
    /// use audio_utils::MonoAudio;
    /// use audio_cleaning::FilteringComparison;
    ///
    /// let before = MonoAudio::new(vec![0.0; 1024], 44100);
    /// let after = MonoAudio::new(vec![0.0; 1024], 44100);
    /// let mut comparison = FilteringComparison::new(before, after);
    /// comparison.compute_spectra();
    /// assert!(comparison.before_spectrum.is_some());
    /// assert!(comparison.after_spectrum.is_some());
    /// ```
    pub fn compute_spectra(&mut self) {
        self.before_spectrum = Some(Spectrum::from_waveform(&self.before.samples));
        self.after_spectrum = Some(Spectrum::from_waveform(&self.after.samples));
    }

    /// Get the magnitude spectra for both before and after audio
    ///
    /// Returns the magnitude spectra (positive frequencies only) for comparison.
    /// Computes the spectra if they haven't been computed yet.
    ///
    /// # Returns
    /// A tuple containing:
    /// - Vector of magnitudes for the before audio
    /// - Vector of magnitudes for the after audio
    ///
    /// # Example
    /// ```
    /// use audio_utils::MonoAudio;
    /// use audio_cleaning::FilteringComparison;
    ///
    /// let before = MonoAudio::new(vec![0.0; 1024], 44100);
    /// let after = MonoAudio::new(vec![0.0; 1024], 44100);
    /// let mut comparison = FilteringComparison::new(before, after);
    /// let (before_mags, after_mags) = comparison.get_magnitude_spectra();
    /// assert_eq!(before_mags.len(), 512); // Half of 1024
    /// assert_eq!(after_mags.len(), 512);
    /// ```
    pub fn get_magnitude_spectra(&mut self) -> (Vec<f32>, Vec<f32>) {
        if self.before_spectrum.is_none() || self.after_spectrum.is_none() {
            self.compute_spectra();
        }
        
        let before_mags = self.before_spectrum.as_ref()
            .map(|s| s.magnitudes())
            .unwrap_or_default();
        let after_mags = self.after_spectrum.as_ref()
            .map(|s| s.magnitudes())
            .unwrap_or_default();
        
        (before_mags, after_mags)
    }

    /// Save both before and after audio to separate WAV files
    ///
    /// # Arguments
    /// * `before_path` - Path where the before audio should be saved
    /// * `after_path` - Path where the after audio should be saved
    ///
    /// # Returns
    /// * `Ok(())` - Successfully saved both files
    /// * `Err(AudioIoError)` - Error saving one of the files
    ///
    /// # Example
    /// ```no_run
    /// use audio_utils::MonoAudio;
    /// use audio_cleaning::FilteringComparison;
    ///
    /// let before = MonoAudio::new(vec![0.0, 1.0, 0.0, -1.0], 44100);
    /// let after = MonoAudio::new(vec![0.0, 0.5, 0.0, -0.5], 44100);
    /// let comparison = FilteringComparison::new(before, after);
    /// comparison.save_audio_pair("before.wav", "after.wav").unwrap();
    /// ```
    pub fn save_audio_pair<P: AsRef<Path>>(&self, before_path: P, after_path: P) -> Result<(), AudioIoError> {
        save_wav(before_path, &self.before)?;
        save_wav(after_path, &self.after)?;
        Ok(())
    }

    /// Get waveform samples for both before and after audio
    ///
    /// Returns references to the sample vectors for easy comparison and visualization.
    ///
    /// # Returns
    /// A tuple containing:
    /// - Reference to before audio samples
    /// - Reference to after audio samples
    ///
    /// # Example
    /// ```
    /// use audio_utils::MonoAudio;
    /// use audio_cleaning::FilteringComparison;
    ///
    /// let before = MonoAudio::new(vec![0.0, 1.0, 0.0, -1.0], 44100);
    /// let after = MonoAudio::new(vec![0.0, 0.5, 0.0, -0.5], 44100);
    /// let comparison = FilteringComparison::new(before, after);
    /// let (before_samples, after_samples) = comparison.get_waveforms();
    /// assert_eq!(before_samples.len(), 4);
    /// assert_eq!(after_samples.len(), 4);
    /// ```
    pub fn get_waveforms(&self) -> (&[f32], &[f32]) {
        (&self.before.samples, &self.after.samples)
    }

    /// Get sample rate (both audio signals should have the same sample rate)
    ///
    /// # Returns
    /// The sample rate in Hz
    pub fn sample_rate(&self) -> u32 {
        self.before.sample_rate
    }
}

/// Create a filtering comparison by applying a filtering function
///
/// This is a convenience function that takes the original audio and a filtering function,
/// applies the filter, and returns a comparison object.
///
/// # Arguments
/// * `audio` - Original audio to filter
/// * `filter_fn` - Function that takes audio and returns filtered audio
///
/// # Returns
/// A FilteringComparison containing the before and after audio
///
/// # Example
/// ```
/// use audio_utils::MonoAudio;
/// use audio_cleaning::{compare_filtering, clean_audio_for_pitch};
///
/// let audio = MonoAudio::new(vec![0.0, 1.0, 0.0, -1.0], 44100);
/// let comparison = compare_filtering(&audio, |a| clean_audio_for_pitch(a, None, None));
/// ```
pub fn compare_filtering<F>(audio: &MonoAudio, filter_fn: F) -> FilteringComparison
where
    F: FnOnce(&MonoAudio) -> MonoAudio,
{
    let before = audio.clone();
    let after = filter_fn(audio);
    FilteringComparison::new(before, after)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filtering_comparison_new() {
        let before = MonoAudio::new(vec![0.0, 1.0, 0.0, -1.0], 44100);
        let after = MonoAudio::new(vec![0.0, 0.5, 0.0, -0.5], 44100);
        let comparison = FilteringComparison::new(before, after);
        
        assert_eq!(comparison.before.samples.len(), 4);
        assert_eq!(comparison.after.samples.len(), 4);
        assert!(comparison.before_spectrum.is_none());
        assert!(comparison.after_spectrum.is_none());
    }

    #[test]
    fn test_compute_spectra() {
        let before = MonoAudio::new(vec![0.0; 1024], 44100);
        let after = MonoAudio::new(vec![0.0; 1024], 44100);
        let mut comparison = FilteringComparison::new(before, after);
        
        comparison.compute_spectra();
        
        assert!(comparison.before_spectrum.is_some());
        assert!(comparison.after_spectrum.is_some());
    }

    #[test]
    fn test_get_magnitude_spectra() {
        let before = MonoAudio::new(vec![0.0; 1024], 44100);
        let after = MonoAudio::new(vec![0.0; 1024], 44100);
        let mut comparison = FilteringComparison::new(before, after);
        
        let (before_mags, after_mags) = comparison.get_magnitude_spectra();
        
        assert_eq!(before_mags.len(), 512); // Half of 1024 (positive frequencies only)
        assert_eq!(after_mags.len(), 512);
    }

    #[test]
    fn test_get_waveforms() {
        let before_samples = vec![0.0, 1.0, 0.0, -1.0];
        let after_samples = vec![0.0, 0.5, 0.0, -0.5];
        let before = MonoAudio::new(before_samples.clone(), 44100);
        let after = MonoAudio::new(after_samples.clone(), 44100);
        let comparison = FilteringComparison::new(before, after);
        
        let (before_wave, after_wave) = comparison.get_waveforms();
        
        assert_eq!(before_wave, &before_samples[..]);
        assert_eq!(after_wave, &after_samples[..]);
    }

    #[test]
    fn test_sample_rate() {
        let before = MonoAudio::new(vec![0.0; 100], 48000);
        let after = MonoAudio::new(vec![0.0; 100], 48000);
        let comparison = FilteringComparison::new(before, after);
        
        assert_eq!(comparison.sample_rate(), 48000);
    }

    #[test]
    fn test_compare_filtering() {
        let audio = MonoAudio::new(vec![1.0, 2.0, 3.0, 4.0], 44100);
        let comparison = compare_filtering(&audio, |a| {
            // Simple "filter" that halves all values
            MonoAudio::new(
                a.samples.iter().map(|&x| x * 0.5).collect(),
                a.sample_rate
            )
        });
        
        assert_eq!(comparison.before.samples, vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(comparison.after.samples, vec![0.5, 1.0, 1.5, 2.0]);
    }
}
