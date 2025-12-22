//! マスク生成

use image::{GrayImage, RgbaImage};

use crate::color::{rgb_to_hsv, Rgb};

/// 画像から指定色のクロマキーマスクを生成
///
/// # Arguments
/// * `image` - 入力画像
/// * `target_color` - クロマキー対象色
/// * `tolerance` - 色の許容範囲 (0.0 - 1.0)
///
/// # Returns
/// グレースケールマスク（白=クロマキー対象、黒=保持）
pub fn create_chroma_mask(image: &RgbaImage, target_color: &Rgb, tolerance: f32) -> GrayImage {
    let (width, height) = image.dimensions();
    let mut mask = GrayImage::new(width, height);

    // ターゲット色をHSVに変換
    let target_hsv = rgb_to_hsv(target_color);

    for (x, y, pixel) in image.enumerate_pixels() {
        let pixel_rgb = Rgb::from_rgba(*pixel);
        let pixel_hsv = rgb_to_hsv(&pixel_rgb);

        // HSV空間での距離を計算
        let is_chroma = pixel_hsv.is_within_tolerance(&target_hsv, tolerance);

        // クロマキー対象は白(255)、保持対象は黒(0)
        let mask_value = if is_chroma { 255 } else { 0 };
        mask.put_pixel(x, y, image::Luma([mask_value]));
    }

    mask
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_green_detection() {
        let mut image = RgbaImage::new(3, 1);
        // 緑、赤、青のピクセル
        image.put_pixel(0, 0, Rgba([0, 255, 0, 255])); // 緑
        image.put_pixel(1, 0, Rgba([255, 0, 0, 255])); // 赤
        image.put_pixel(2, 0, Rgba([0, 0, 255, 255])); // 青

        let target = Rgb::new(0, 255, 0);
        let mask = create_chroma_mask(&image, &target, 0.3);

        // 緑のピクセルだけがマスクされる
        assert_eq!(mask.get_pixel(0, 0).0[0], 255); // 緑はクロマキー対象
        assert_eq!(mask.get_pixel(1, 0).0[0], 0); // 赤は保持
        assert_eq!(mask.get_pixel(2, 0).0[0], 0); // 青は保持
    }

    #[test]
    fn test_tolerance() {
        let mut image = RgbaImage::new(2, 1);
        image.put_pixel(0, 0, Rgba([0, 255, 0, 255])); // 完全な緑
        image.put_pixel(1, 0, Rgba([50, 200, 50, 255])); // 暗めの緑

        let target = Rgb::new(0, 255, 0);

        // 低い許容範囲
        let mask_low = create_chroma_mask(&image, &target, 0.1);
        assert_eq!(mask_low.get_pixel(0, 0).0[0], 255); // 完全な緑は検出

        // 高い許容範囲
        let mask_high = create_chroma_mask(&image, &target, 0.5);
        assert_eq!(mask_high.get_pixel(0, 0).0[0], 255); // 完全な緑
        assert_eq!(mask_high.get_pixel(1, 0).0[0], 255); // 暗めの緑も検出
    }
}

