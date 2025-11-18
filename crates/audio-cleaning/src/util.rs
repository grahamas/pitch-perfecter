pub fn rms(signal: &[f32]) -> Option<f32> {
    if signal.is_empty() {
        return None;
    }
    let sum_squares: f32 = signal.iter().map(|&x| x * x).sum();
    Some((sum_squares / signal.len() as f32).sqrt())
}

pub fn mean(data: &[f32]) -> Option<f32> {
    let sum = data.iter().sum::<f32>();
    let count = data.len();

    match count {
        positive if positive > 0 => Some(sum / count as f32),
        _ => None,
    }
}

pub fn mean_std_deviation(data: &[f32]) -> Option<(f32, f32)> {
    let mean_value = mean(data)?;
    let variance = data
        .iter()
        .map(|&value| {
            let diff = mean_value - value;
            diff * diff
        })
        .sum::<f32>()
        / data.len() as f32;
    let std_dev = variance.sqrt();
    Some((mean_value, std_dev))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rms_empty() {
        assert_eq!(rms(&[]), None);
    }

    #[test]
    fn test_rms_known() {
        let sig = [3.0, 4.0];
        let expected = ((3.0f32 * 3.0 + 4.0 * 4.0) / 2.0).sqrt();
        assert!((rms(&sig).unwrap() - expected).abs() < 1e-6);
    }

    #[test]
    fn test_mean_empty() {
        assert_eq!(mean(&[]), None);
    }

    #[test]
    fn test_mean_known() {
        let data = [1.0, 2.0, 3.0];
        assert!((mean(&data).unwrap() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_mean_std_deviation_empty() {
        assert_eq!(mean_std_deviation(&[]), None);
    }

    #[test]
    fn test_mean_std_deviation_known() {
        let data = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let (mean, stddev) = mean_std_deviation(&data).unwrap();
        assert!((mean - 5.0).abs() < 1e-6);
        assert!((stddev - 2.0).abs() < 1e-6);
    }
}
