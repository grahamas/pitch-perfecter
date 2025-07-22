use rustfft::num_complex::Complex;
use rustfft::{FftPlanner, FftDirection};

/// Struct representing a computed spectrum, with ability to invert (IFFT) back to time domain
pub struct Spectrum {
    pub complex: Vec<Complex<f32>>, // Full complex spectrum (not just magnitudes)
    pub n: usize,                   // FFT size
}

impl Spectrum {
    /// Compute the full spectrum of a real-valued signal (returns Spectrum struct)
    pub fn from_waveform(signal: &[f32]) -> Self {
        // FIXME save the planner for reuse
        let n_fft = signal.len();
        let spectrum = compute_spectrum(signal, n_fft);
        Self { complex: spectrum, n: n_fft }
    }

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

fn compute_spectrum(signal: &[f32], n_fft: usize) -> Vec<Complex<f32>> {
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(n_fft);
    let mut buffer: Vec<Complex<f32>> = signal.iter().map(|&x| Complex::new(x, 0.0)).collect();
    fft.process(&mut buffer);
    return buffer;
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

/// Compute the spectrogram of a real-valued signal
/// - window_size: number of samples per FFT window
/// - step_size: number of samples to step between windows
/// Returns: Vec of spectra (each spectrum is Vec<f32>)
fn compute_spectrogram(signal: &[f32], window_size: usize, step_size: usize) -> Vec<Vec<f32>> {
    let mut result = Vec::new();
    let mut i = 0;
    while i + window_size <= signal.len() {
        let window = &signal[i..i+window_size];
        let spectrum = Spectrum::from_waveform(window).magnitudes();
        // Only keep the positive frequencies (first half of the spectrum)
        let spectrum = spectrum[..window_size/2].to_vec();
        result.push(spectrum);
        i += step_size;
    }
    result
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


#[cfg(test)]
mod tests {
    use super::*;
    use rustfft::num_complex::Complex;

    #[test]
    fn test_spectrum_from_waveform_and_magnitudes() {
        // Simple sine wave: only one frequency bin should be nonzero
        let n = 8;
        let mut signal = vec![0.0f32; n];
        for i in 0..n {
            signal[i] = (2.0 * std::f32::consts::PI * i as f32 / n as f32).sin();
        }
        let spectrum = Spectrum::from_waveform(&signal);
        assert_eq!(spectrum.complex.len(), n);
        let mags = spectrum.magnitudes();
        assert_eq!(mags.len(), n/2);
        // Magnitudes should be non-negative
        assert!(mags.iter().all(|&m| m >= 0.0));
    }

    #[test]
    fn test_spectrum_to_time_domain_identity() {
        // FFT then IFFT should recover original (within tolerance)
        let signal = vec![1.0, 0.0, -1.0, 0.0];
        let spectrum = Spectrum::from_waveform(&signal);
        let recovered = spectrum.to_time_domain();
        assert_eq!(recovered.len(), signal.len());
        for (a, b) in recovered.iter().zip(signal.iter()) {
            assert!((a - b).abs() < 1e-5);
        }
    }

    #[test]
    fn test_spectrum_get() {
        let signal = vec![1.0, 2.0, 3.0, 4.0];
        let spectrum = Spectrum::from_waveform(&signal);
        assert!(spectrum.get(0).is_some());
        assert!(spectrum.get(signal.len()).is_none());
    }

    #[test]
    fn test_spectrogram_from_waveform() {
        let signal = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let config = SpectrogramConfig { window_size: 4, step_size: 2 };
        let spec = Spectrogram::from_waveform(&signal, config);
        // With window_size=4, step_size=2, expect 3 windows
        assert_eq!(spec.n_time_steps(), 3);
        // Each spectrum should have window_size/2 bins
        assert_eq!(spec.n_freq_bins(), 2);
    }
}

