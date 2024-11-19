use image::GrayImage;

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

    camera.set_exposure(10000, true).expect("setting exposure");
    camera.set_gain(200, true).expect("setting gain");

    camera
        .set_image_size(
            camera.properties().max_width,
            camera.properties().max_height,
        )
        .expect("setting image size");

    let mut i = 0;

    camera
        .stream(Some(1000), |camera, buffer| {
            let img: GrayImage = image::ImageBuffer::from_vec(
                camera.properties().max_width,
                camera.properties().max_height,
                buffer.iter().copied().collect::<Vec<u8>>(),
            )
            .expect("converting to image buffer");

            img.save(format!("camera_frame_{}.png", i))
                .expect("saving to file failed");

            i += 1;
            if i == 10 {
                return false;
            }
            true
        })
        .expect("stream failed");

    camera.close().unwrap();
}
