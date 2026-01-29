//! Latency Tracking Module
//!
//! This module provides types for tracking audio processing latency from input to output.
//! It helps identify performance bottlenecks in the audio processing pipeline.

use std::time::{Duration, Instant};

/// Metrics for tracking latency through the audio processing pipeline
#[derive(Debug, Clone)]
pub struct LatencyMetrics {
    /// Timestamp when the audio callback was invoked (input device timestamp)
    pub callback_timestamp: Option<Instant>,
    
    /// Timestamp when audio processing started
    pub processing_start: Option<Instant>,
    
    /// Timestamp when audio processing completed
    pub processing_end: Option<Instant>,
    
    /// Input device latency as reported by the audio driver (if available)
    /// This is the time from when audio was captured by hardware to when the callback was invoked
    pub input_device_latency: Option<Duration>,
}

impl LatencyMetrics {
    /// Create a new empty latency metrics instance
    pub fn new() -> Self {
        Self {
            callback_timestamp: None,
            processing_start: None,
            processing_end: None,
            input_device_latency: None,
        }
    }
    
    /// Create metrics with a callback timestamp
    pub fn with_callback_timestamp(timestamp: Instant) -> Self {
        Self {
            callback_timestamp: Some(timestamp),
            processing_start: None,
            processing_end: None,
            input_device_latency: None,
        }
    }
    
    /// Record the start of processing
    pub fn mark_processing_start(&mut self) {
        self.processing_start = Some(Instant::now());
    }
    
    /// Record the end of processing
    pub fn mark_processing_end(&mut self) {
        self.processing_end = Some(Instant::now());
    }
    
    /// Set the input device latency from audio driver info
    pub fn set_input_device_latency(&mut self, latency: Duration) {
        self.input_device_latency = Some(latency);
    }
    
    /// Calculate the processing time (cleaning + pitch detection)
    pub fn processing_duration(&self) -> Option<Duration> {
        match (self.processing_start, self.processing_end) {
            (Some(start), Some(end)) => Some(end.duration_since(start)),
            _ => None,
        }
    }
    
    /// Calculate the total latency from callback to processing completion
    pub fn total_latency(&self) -> Option<Duration> {
        match (self.callback_timestamp, self.processing_end) {
            (Some(callback), Some(end)) => Some(end.duration_since(callback)),
            _ => None,
        }
    }
    
    /// Get the total end-to-end latency including input device latency
    pub fn end_to_end_latency(&self) -> Option<Duration> {
        match (self.total_latency(), self.input_device_latency) {
            (Some(total), Some(device)) => Some(total + device),
            (Some(total), None) => Some(total),
            (None, Some(device)) => Some(device),
            (None, None) => None,
        }
    }
}

impl Default for LatencyMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_latency_metrics_creation() {
        let metrics = LatencyMetrics::new();
        assert!(metrics.callback_timestamp.is_none());
        assert!(metrics.processing_start.is_none());
        assert!(metrics.processing_end.is_none());
        assert!(metrics.input_device_latency.is_none());
    }
    
    #[test]
    fn test_latency_metrics_with_callback() {
        let now = Instant::now();
        let metrics = LatencyMetrics::with_callback_timestamp(now);
        assert!(metrics.callback_timestamp.is_some());
        assert_eq!(metrics.callback_timestamp.unwrap(), now);
    }
    
    #[test]
    fn test_processing_duration() {
        let mut metrics = LatencyMetrics::new();
        
        // No duration when not set
        assert!(metrics.processing_duration().is_none());
        
        // Set start and end with a delay
        metrics.mark_processing_start();
        thread::sleep(Duration::from_millis(10));
        metrics.mark_processing_end();
        
        let duration = metrics.processing_duration();
        assert!(duration.is_some());
        assert!(duration.unwrap() >= Duration::from_millis(10));
    }
    
    #[test]
    fn test_total_latency() {
        let mut metrics = LatencyMetrics::with_callback_timestamp(Instant::now());
        
        thread::sleep(Duration::from_millis(10));
        metrics.mark_processing_end();
        
        let latency = metrics.total_latency();
        assert!(latency.is_some());
        assert!(latency.unwrap() >= Duration::from_millis(10));
    }
    
    #[test]
    fn test_end_to_end_latency() {
        let mut metrics = LatencyMetrics::with_callback_timestamp(Instant::now());
        metrics.set_input_device_latency(Duration::from_millis(5));
        
        thread::sleep(Duration::from_millis(10));
        metrics.mark_processing_end();
        
        let e2e = metrics.end_to_end_latency();
        assert!(e2e.is_some());
        // Should be at least 10ms (our sleep) + 5ms (device latency)
        assert!(e2e.unwrap() >= Duration::from_millis(15));
    }
    
    #[test]
    fn test_end_to_end_with_only_device_latency() {
        let mut metrics = LatencyMetrics::new();
        metrics.set_input_device_latency(Duration::from_millis(5));
        
        let e2e = metrics.end_to_end_latency();
        assert_eq!(e2e, Some(Duration::from_millis(5)));
    }
}
