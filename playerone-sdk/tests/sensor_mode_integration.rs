//! Integration tests for sensor-mode (Dual Sampling) APIs.
//!
//! **Requires a real Player One camera connected via USB.**
//!
//! These tests are `#[ignore]`-d by default. Run explicitly with:
//! ```sh
//! cargo test --test sensor_mode_integration -- --ignored
//! ```

use playerone_sdk::{Camera, Error, SensorMode};

fn find_mode_by_keywords<'a>(modes: &'a [SensorMode], keywords: &[&str]) -> Option<&'a SensorMode> {
    modes.iter().find(|m| {
        let name = m.name.to_lowercase();
        keywords.iter().any(|k| name.contains(k))
    })
}

fn open_first_camera() -> playerone_sdk::Camera {
    let cameras = Camera::all_cameras();
    assert!(!cameras.is_empty(), "No Player One cameras found — connect a camera via USB before running this test");
    cameras.into_iter().next().unwrap().open().expect("failed to open camera")
}

#[test]
#[ignore]
fn sensor_modes_are_enumerated() {
    let camera = open_first_camera();
    let modes = camera.sensor_modes();

    // We can't assert the count because it's camera-dependent, but if the
    // camera supports dual sampling it should have ≥ 2 modes.
    if modes.is_empty() {
        println!(
            "Camera '{}' does not support dual sampling — skipping",
            camera.properties().camera_model_name
        );
        return;
    }

    assert!(modes.len() >= 2, "Expected at least 2 sensor modes, got {}", modes.len());

    for mode in modes {
        assert!(!mode.name.is_empty(), "Sensor mode name should not be empty");
        println!("  [{}] {} — {}", mode.index, mode.name, mode.description);
    }
}

#[test]
#[ignore]
fn switch_to_normal_then_lrn() {
    let mut camera = open_first_camera();
    let modes = camera.sensor_modes().to_vec();

    if modes.is_empty() {
        println!(
            "Camera '{}' does not support dual sampling — skipping",
            camera.properties().camera_model_name
        );
        return;
    }

    // Find Normal mode
    let normal = find_mode_by_keywords(&modes, &["normal"])
        .expect("No 'Normal' mode found on camera");
    let normal_index = normal.index;

    // Find LRN mode (different cameras use different labels)
    let lrn = find_mode_by_keywords(&modes, &["lrn", "low readout", "low read", "low noise"])
        .expect("No LRN/Low Noise mode found on camera");
    let lrn_index = lrn.index;

    // Switch to Normal
    camera
        .set_sensor_mode(normal_index)
        .expect("failed to switch to Normal mode");
    let current = camera.sensor_mode().expect("failed to read sensor mode");
    assert_eq!(current, normal_index, "Expected Normal mode index {normal_index}, got {current}");

    // Switch to LRN
    camera
        .set_sensor_mode(lrn_index)
        .expect("failed to switch to LRN mode");
    let current = camera.sensor_mode().expect("failed to read sensor mode");
    assert_eq!(current, lrn_index, "Expected LRN mode index {lrn_index}, got {current}");
}

#[test]
#[ignore]
fn set_sensor_mode_out_of_bounds() {
    let mut camera = open_first_camera();
    let modes = camera.sensor_modes();

    if modes.is_empty() {
        println!(
            "Camera '{}' does not support dual sampling — skipping",
            camera.properties().camera_model_name
        );
        return;
    }

    let invalid_index = modes.len() as u32 + 10;
    let result = camera.set_sensor_mode(invalid_index);
    assert!(
        matches!(result, Err(Error::OutOfBounds)),
        "Expected OutOfBounds error for index {invalid_index}, got {result:?}"
    );
}

#[test]
#[ignore]
fn display_impl_for_sensor_mode() {
    let camera = open_first_camera();
    let modes = camera.sensor_modes();

    if modes.is_empty() {
        println!(
            "Camera '{}' does not support dual sampling — skipping",
            camera.properties().camera_model_name
        );
        return;
    }

    for mode in modes {
        let displayed = format!("{}", mode);
        assert_eq!(displayed, mode.name, "Display should output the mode name");
        // Verify trimming — no leading/trailing whitespace
        assert_eq!(displayed, displayed.trim(), "Mode name should be trimmed");
    }
}
