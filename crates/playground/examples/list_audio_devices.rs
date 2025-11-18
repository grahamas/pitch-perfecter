use audio_utils::recording::list_input_devices;

/// Simple example that lists all available audio input devices.
///
/// This is useful for:
/// - Checking if your microphone is detected
/// - Seeing what audio inputs are available on your system
/// - Verifying audio hardware configuration
///
/// Usage:
/// ```bash
/// cargo run --package playground --example list_audio_devices
/// ```
fn main() {
    println!("=== Audio Input Devices ===\n");
    
    match list_input_devices() {
        Ok(devices) => {
            if devices.is_empty() {
                println!("No input devices found!");
                println!("\nPossible reasons:");
                println!("  - No microphone is connected");
                println!("  - Audio drivers are not installed");
                println!("  - Running in a headless environment (e.g., CI)");
            } else {
                println!("Found {} input device{}:", 
                    devices.len(), 
                    if devices.len() == 1 { "" } else { "s" }
                );
                println!();
                
                for (i, device) in devices.iter().enumerate() {
                    let marker = if device.is_default { " ★ (default)" } else { "" };
                    println!("  {}. {}{}", i + 1, device.name, marker);
                }
                
                println!();
                println!("★ The default device will be used for recording");
            }
        }
        Err(e) => {
            println!("Error listing devices: {}", e);
            println!("\nThis might indicate:");
            println!("  - Audio system is not available");
            println!("  - Permissions issue");
        }
    }
}
