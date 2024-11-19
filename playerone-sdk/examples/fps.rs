use std::time::Instant;

use playerone_sdk::Camera;
use playerone_sdk::ImageFormat;

pub fn main() {
    let camera_description = Camera::all_cameras()
        .into_iter()
        .nth(0)
        .expect("No cameras found");

    let mut camera = camera_description.open().expect("opening camera");

    println!("camera properties:\n{:#?}\n", camera.properties());

    let bounds = camera.config_bounds();
    println!("camera bounds:\n{:#?}\n", bounds);

    camera
        .set_image_format(ImageFormat::RAW8)
        .expect("setting image format");

    camera.set_exposure(500, true).expect("setting exposure");
    camera.set_gain(200, true).expect("setting gain");
    camera
        .set_usb_bandwidth_limit(bounds.usb_bandwidth_limit.max)
        .expect("setting usb bandwidth limit");

    camera
        .set_image_size(
            camera.properties().max_width,
            camera.properties().max_height,
        )
        .expect("setting image size");

    let mut last_time = Instant::now();

    camera
        .stream(Some(1000), |camera, buffer| {
            print!("FPS: {:.2}    \r", 1.0 / last_time.elapsed().as_secs_f64());

            last_time = Instant::now();

            true
        })
        .expect("stream failed");
}
