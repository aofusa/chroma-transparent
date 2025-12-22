//! アルファチャンネル処理

use image::{GrayImage, RgbaImage};

/// マスクからアルファチャンネルを生成（反転）
/// マスクの白(255)→透明(0)、黒(0)→不透明(255)
pub fn create_alpha_from_mask(mask: &GrayImage) -> GrayImage {
    let (width, height) = mask.dimensions();
    let mut alpha = GrayImage::new(width, height);

    for (x, y, pixel) in mask.enumerate_pixels() {
        // 反転: 255 - value
        let alpha_value = 255 - pixel.0[0];
        alpha.put_pixel(x, y, image::Luma([alpha_value]));
    }

    alpha
}

/// 画像にアルファチャンネルを適用
pub fn apply_alpha(image: &mut RgbaImage, alpha: &GrayImage) {
    let (width, height) = image.dimensions();

    for y in 0..height {
        for x in 0..width {
            if x < alpha.width() && y < alpha.height() {
                let alpha_value = alpha.get_pixel(x, y).0[0];
                let pixel = image.get_pixel_mut(x, y);
                pixel.0[3] = alpha_value;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_create_alpha_from_mask() {
        let mut mask = GrayImage::new(2, 1);
        mask.put_pixel(0, 0, image::Luma([255])); // クロマキー対象
        mask.put_pixel(1, 0, image::Luma([0])); // 保持対象

        let alpha = create_alpha_from_mask(&mask);

        assert_eq!(alpha.get_pixel(0, 0).0[0], 0); // 透明
        assert_eq!(alpha.get_pixel(1, 0).0[0], 255); // 不透明
    }

    #[test]
    fn test_apply_alpha() {
        let mut image = RgbaImage::new(2, 1);
        image.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        image.put_pixel(1, 0, Rgba([0, 255, 0, 255]));

        let mut alpha = GrayImage::new(2, 1);
        alpha.put_pixel(0, 0, image::Luma([128]));
        alpha.put_pixel(1, 0, image::Luma([255]));

        apply_alpha(&mut image, &alpha);

        assert_eq!(image.get_pixel(0, 0).0[3], 128);
        assert_eq!(image.get_pixel(1, 0).0[3], 255);
    }
}

