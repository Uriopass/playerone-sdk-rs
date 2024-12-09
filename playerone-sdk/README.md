# Player One SDK

Wrapper for the [Player One Camera SDK](https://player-one-astronomy.com/service/software).

See `examples` for usage.

Here's a minimal one for reference:

```rust
pub fn main() {
    let camera_description = playerone_sdk::Camera::all_cameras()
        .into_iter()
        .nth(0)
        .expect("No cameras found");

    let mut camera = camera_description.open().expect("opening camera");
    println!("camera properties:\n{:#?}\n", camera.properties());

    let bounds = camera.config_bounds();
    println!("camera bounds:\n{:#?}\n", bounds);

    camera
        .set_image_format(playerone_sdk::ImageFormat::RAW8)
        .expect("setting image format");

    camera.set_exposure(10000, true).expect("setting exposure");
    camera.set_gain(200, true).expect("setting gain");

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

    let (w, h) = camera.image_size();

    let img: image::GrayImage =
        image::ImageBuffer::from_vec(w, h, buffer).expect("converting to image buffer");

    img.save("camera_frame.png").expect("saving to file failed");
}
```