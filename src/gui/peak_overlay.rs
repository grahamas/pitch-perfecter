use egui::{epaint::{Color32, Shape}, Rect, Painter, pos2};

/// Draws the moving peak overlay as yellow circles on the given painter.
/// - `peak_indices`: vector of frequency bin indices (one per time step)
/// - `rect`: the drawing area
/// - `n_freq`: number of frequency bins
/// - `n_time`: number of time bins
pub fn draw_peak_overlay(peak_indices: &[usize], rect: Rect, painter: &Painter, n_freq: usize, n_time: usize) {
    if peak_indices.len() > 1 {
        for (t, &freq_bin) in peak_indices.iter().enumerate() {
            if freq_bin == 0 || freq_bin >= n_freq-1 { continue; }
            let x = t as f32 / n_time as f32;
            let y = 1.0 - (freq_bin as f32 / n_freq as f32); // flip vertically
            let mapped = pos2(rect.left() + x * rect.width(), rect.top() + y * rect.height());
            painter.add(Shape::circle_filled(mapped, 3.0, Color32::YELLOW));
        }
    }
}
