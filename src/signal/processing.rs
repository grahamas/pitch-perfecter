/// FIXME  file needs refinement after AI

//! Signal processing utilities: spectrum and spectrogram
use rustfft::{FftPlanner, num_complex::Complex, FftDirection};

/// Struct representing a computed spectrum, with ability to invert (IFFT) back to time domain
pub struct Spectrum {
    pub complex: Vec<Complex<f32>>, // Full complex spectrum (not just magnitudes)
    pub n: usize,                   // FFT size
}

impl Spectrum {
    /// Get the magnitude spectrum (positive frequencies only)
    pub fn magnitudes(&self) -> Vec<f32> {
        self.complex[..self.n/2].iter().map(|c| c.norm()).collect()
    }

    /// Invert the spectrum back to the time domain (real part only)
    pub fn to_time_domain(&self) -> Vec<f32> {
        let mut buffer = self.complex.clone();
        let mut planner = FftPlanner::<f32>::new();
        let ifft = planner.plan_fft(self.n, FftDirection::Inverse);
        ifft.process(&mut buffer);
        buffer.iter().map(|c| c.re / self.n as f32).collect()
    }

    // Get the complex value at index i
    pub fn get(&self, i: usize) -> Option<&Complex<f32>> {
        self.complex.get(i)
    }
}

/// Compute the full spectrum of a real-valued signal (returns Spectrum struct)
pub fn compute_spectrum(signal: &[f32]) -> Spectrum {
    let n = signal.len();
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(n);
    let mut buffer: Vec<Complex<f32>> = signal.iter().map(|&x| Complex::new(x, 0.0)).collect();
    fft.process(&mut buffer);
    Spectrum { complex: buffer, n }
}

// TODO add frequency axis
pub struct Spectrogram {
    pub spectra: Vec<Vec<f32>>, // Vec of spectra (each spectrum is Vec<f32>)
    pub window_size: usize,      // Size of each FFT window
    pub step_size: usize,        // Step size between windows
}

impl Spectrogram {
    pub fn from_waveform(signal: &[f32], config: SpectrogramConfig) -> Self {
        let spectra = compute_spectrogram(signal, config.window_size, config.step_size);
        Self {
            spectra,
            window_size: config.window_size,
            step_size: config.step_size,
        }
    }
    /// Get the number of time steps in the spectrogram
    pub fn n_time_steps(&self) -> usize {
        self.spectra.len()
    }
    /// Get the number of frequency bins in each spectrum
    pub fn n_freq_bins(&self) -> usize {
        if self.spectra.is_empty() {
            0
        } else {
            self.spectra[0].len()
        }
    }
}

pub struct SpectrogramConfig {
    pub window_size: usize, // Number of samples per FFT window
    pub step_size: usize,   // Number of samples to step between windows
}

impl SpectrogramConfig { 
    pub fn default() -> Self {
        Self {
            window_size: 1024,
            step_size: 256,
        }
    }
}

/// Compute the spectrogram of a real-valued signal
/// - window_size: number of samples per FFT window
/// - step_size: number of samples to step between windows
/// Returns: Vec of spectra (each spectrum is Vec<f32>)
pub fn compute_spectrogram(signal: &[f32], window_size: usize, step_size: usize) -> Vec<Vec<f32>> {
    let mut result = Vec::new();
    let mut i = 0;
    while i + window_size <= signal.len() {
        let window = &signal[i..i+window_size];
        let spectrum = compute_spectrum(window).magnitudes();
        // Only keep the positive frequencies (first half of the spectrum)
        let spectrum = spectrum[..window_size/2].to_vec();
        result.push(spectrum);
        i += step_size;
    }
    result
}

/// Find the index and value of the peak in a spectrum
pub fn find_spectrum_peak(spectrum: &[f32]) -> Option<(usize, f32)> {
    spectrum
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i, &v)| (i, v))
}

/// For a spectrogram (Vec of spectra), return a Vec of peak indices (one per time window)
pub fn detect_moving_peak(spectrogram: &[Vec<f32>]) -> Vec<usize> {
    spectrogram
        .iter()
        .map(|spec| find_spectrum_peak(spec).map(|(i, _)| i).unwrap_or(0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    // Helper: generate a sine wave
    fn sine_wave(freq: f32, sample_rate: f32, len: usize) -> Vec<f32> {
        (0..len)
            .map(|i| (2.0 * PI * freq * i as f32 / sample_rate).sin())
            .collect()
    }

    #[test]
    fn test_spectrum_detects_sine() {
        let sample_rate = 1024.0;
        let freq = 128.0;
        let len = 1024;
        let signal = sine_wave(freq, sample_rate, len);
        let spectrum = compute_spectrum(&signal);
        // The peak should be at bin k = freq * N / sample_rate
        let k = (freq * len as f32 / sample_rate).round() as usize;
        let max_bin = spectrum.magnitudes().iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
        assert_eq!(k, max_bin);
    }

    #[test]
    fn test_spectrogram_detects_two_tones() {
        let sample_rate = 1000.0;
        let len = 1000;
        let mut signal = sine_wave(100.0, sample_rate, len);
        let signal2 = sine_wave(200.0, sample_rate, len);
        for (a, b) in signal.iter_mut().zip(signal2.iter()) {
            *a += b;
        }
        let spec = compute_spectrum(&signal);
        let k1 = (100.0 * len as f32 / sample_rate).round() as usize;
        let k2 = (200.0 * len as f32 / sample_rate).round() as usize;
        let max1 = spec.magnitudes()[k1];
        let max2 = spec.magnitudes()[k2];
        // Both peaks should be prominent
        assert!(max1 > 0.5 * spec.magnitudes().iter().cloned().fold(0.0, f32::max));
        assert!(max2 > 0.5 * spec.magnitudes().iter().cloned().fold(0.0, f32::max));
    }

    #[test]
    fn test_find_spectrum_peak() {
        let spectrum = vec![0.1, 0.5, 2.0, 0.3, 0.2];
        let peak = find_spectrum_peak(&spectrum);
        assert_eq!(peak, Some((2, 2.0)));
    }

    #[test]
    fn test_detect_moving_peak() {
        let spectrogram = vec![
            vec![0.0, 1.0, 0.5],
            vec![0.2, 0.3, 0.9],
            vec![0.7, 0.1, 0.2],
        ];
        let peaks = detect_moving_peak(&spectrogram);
        assert_eq!(peaks, vec![1, 2, 0]);
    }
}
