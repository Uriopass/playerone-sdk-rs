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

    camera
        .set_image_size(
            camera.properties().max_width,
            camera.properties().max_height,
        )
        .expect("setting image size");

    let mut buffer = camera.create_image_buffer();

    camera
        .capture(&mut buffer, Some(1000))
        .expect("getting frame");

    let img: GrayImage = image::ImageBuffer::from_vec(
        camera.properties().max_width,
        camera.properties().max_height,
        buffer,
    )
    .expect("converting to image buffer");

    img.save("camera_frame.png").expect("saving to file failed");
}
