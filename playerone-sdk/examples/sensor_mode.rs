//! Lists sensor modes for the first connected camera and, if at least two
//! modes are available, switches to the one whose name contains "LRN"
//! (Low Readout Noise). Skip if the camera does not support mode selection.

pub fn main() {
    let camera_description = playerone_sdk::Camera::all_cameras()
        .into_iter()
        .next()
        .expect("No cameras found");

    let mut camera = camera_description.open().expect("opening camera");

    let modes = camera.sensor_modes().expect("enumerating sensor modes");
    if modes.is_empty() {
        println!(
            "{} does not support sensor mode selection",
            camera.properties().camera_model_name
        );
        return;
    }

    println!("Available sensor modes:");
    for mode in &modes {
        println!("  [{}] {} - {}", mode.index, mode.name, mode.description);
    }

    let current_idx = camera.sensor_mode().expect("reading current sensor mode");
    let current_name = modes
        .iter()
        .find(|m| m.index == current_idx)
        .map(|m| m.name.as_str())
        .unwrap_or("unknown");
    println!("Current: {} (index {})", current_name, current_idx);

    // Different cameras label the low-readout-noise mode differently:
    // Uranus-C Pro → "LRN", Ares-C PRO → "Low Noise", etc.
    if let Some(lrn) = modes.iter().find(|m| {
        let name = m.name.to_lowercase();
        name.contains("lrn")
            || name.contains("low readout")
            || name.contains("low read")
            || name.contains("low noise")
    }) {
        println!("Switching to: {}", lrn);
        camera
            .set_sensor_mode(lrn.index)
            .expect("setting sensor mode");
    }
}
