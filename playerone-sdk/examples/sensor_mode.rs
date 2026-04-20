//! Lists sensor modes for the first connected camera and, if at least two
//! modes are available, switches to the one whose name contains "LRN"
//! (Low Readout Noise). Skip if the camera does not support mode selection.

pub fn main() {
    let camera_description = playerone_sdk::Camera::all_cameras()
        .into_iter()
        .next()
        .expect("No cameras found");

    let mut camera = camera_description.open().expect("opening camera");

    let modes = camera.sensor_modes().to_vec();
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

    let current = camera.sensor_mode().expect("reading current sensor mode");
    println!("Current: {}", current);

    if let Some(lrn) = modes.iter().find(|m| m.name.to_lowercase().contains("lrn")) {
        println!("Switching to: {}", lrn.name);
        camera
            .set_sensor_mode(lrn.index)
            .expect("setting sensor mode");
    }
}
