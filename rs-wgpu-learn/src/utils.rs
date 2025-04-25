use image::DynamicImage;
// 使用 `reqwest` 和 `image` 库从给定的 URL 加载图像数据，并返回 `DynamicImage` 类型的结果
pub fn load_image_data(url: &str) -> anyhow::Result<DynamicImage> {
    // 通过 `reqwest::blocking::get` 获取 URL 的数据并转换为字节
    let data = reqwest::blocking::get(url)?.bytes()?;
    // 使用 `image::load_from_memory` 从字节数据加载图像
    Ok(image::load_from_memory(&data)?)
}

// 计算图像在屏幕上的缩放比例，返回一个包含宽度和高度缩放比例的数组
pub fn calc_scale(image: [f32; 2], screen: [f32; 2]) -> [f32; 2] {
    // 解构图像和屏幕的宽度和高度
    let [width, height] = image;
    let [screen_width, screen_height] = screen;

    // 计算图像和屏幕的宽高比
    let image_ratio = width / height;
    let screen_ratio = screen_width / screen_height;

    // 根据宽高比调整缩放比例
    if image_ratio > screen_ratio {
        [1.0, screen_ratio / image_ratio] // 图像宽度占满屏幕，高度按比例缩放
    } else {
        [image_ratio / screen_ratio, 1.0] // 图像高度占满屏幕，宽度按比例缩放
    }
}
