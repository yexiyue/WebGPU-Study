use image::DynamicImage;
pub fn load_image_data(url: &str) -> anyhow::Result<DynamicImage> {
    let data = reqwest::blocking::get(url)?.bytes()?;
    Ok(image::load_from_memory(&data)?)
}

pub fn calc_scale(image: [f32; 2], screen: [f32; 2]) -> [f32; 2] {
    let [width, height] = image;
    let [screen_width, screen_height] = screen;

    let image_ratio = width / height;
    let screen_ratio = screen_width / screen_height;

    if image_ratio > screen_ratio {
        [1.0, screen_ratio / image_ratio]
    } else {
        [image_ratio / screen_ratio, 1.0]
    }
}
