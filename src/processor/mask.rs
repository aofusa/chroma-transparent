//! マスク生成（最適化版）
//!
//! 改善案A: Rayon並列処理
//! 改善案B: RGB距離による事前フィルタリング
//! 改善案D: バッファ直接操作
//! 改善案F: LUTによるHSV変換高速化

use image::{GrayImage, RgbaImage};
use rayon::prelude::*;

use crate::color::{rgb_distance_squared, rgb_to_hsv, HsvLut, Rgb};

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

    // LUTを初期化（改善案F）
    let lut = HsvLut::new();
    let target_hsv = rgb_to_hsv(target_color);

    // RGB事前フィルタリングの閾値（改善案B）
    // tolerance * 2 の距離を超えたら明らかに異なる色
    let rgb_threshold_squared = ((tolerance * 2.0 * 441.67) as u32).pow(2);

    // 入力バッファを取得（改善案D）
    let src = image.as_raw();

    // 並列処理でマスク生成（改善案A）
    let mask_data: Vec<u8> = (0..height)
        .into_par_iter()
        .flat_map(|y| {
            let mut row = Vec::with_capacity(width as usize);
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                let r = src[idx];
                let g = src[idx + 1];
                let b = src[idx + 2];

                let pixel_rgb = Rgb::new(r, g, b);

                // RGB距離で事前フィルタリング（改善案B）
                // 明らかに異なる色は詳細判定をスキップ
                let is_chroma = if rgb_distance_squared(&pixel_rgb, target_color)
                    > rgb_threshold_squared
                {
                    false
                } else {
                    // LUTでHSV取得（改善案F）
                    let pixel_hsv = lut.get(&pixel_rgb);
                    pixel_hsv.is_within_tolerance(&target_hsv, tolerance)
                };

                row.push(if is_chroma { 255 } else { 0 });
            }
            row
        })
        .collect();

    GrayImage::from_raw(width, height, mask_data).expect("Failed to create mask image")
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

    #[test]
    fn test_large_image() {
        // 並列処理のテスト用大きな画像
        let mut image = RgbaImage::new(1000, 1000);
        for y in 0..1000 {
            for x in 0..1000 {
                if x < 500 {
                    image.put_pixel(x, y, Rgba([0, 255, 0, 255])); // 左半分は緑
                } else {
                    image.put_pixel(x, y, Rgba([255, 0, 0, 255])); // 右半分は赤
                }
            }
        }

        let target = Rgb::new(0, 255, 0);
        let mask = create_chroma_mask(&image, &target, 0.3);

        // 左半分は白、右半分は黒
        assert_eq!(mask.get_pixel(0, 0).0[0], 255);
        assert_eq!(mask.get_pixel(999, 0).0[0], 0);
    }
}
