//! マスク生成（最適化版）
//!
//! 改善案A: Rayon並列処理（parallel feature有効時）
//! 改善案B: RGB距離による事前フィルタリング
//! 改善案D: バッファ直接操作
//! 改善案F: LUTによるHSV変換高速化

use image::{GrayImage, RgbaImage};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use crate::color::{
    rgb_distance_squared, rgb_to_hsv, rgb_to_lab, rgb_to_lch, rgb_to_yuv, ColorSpace, Hsv, HsvLut,
    Lab, Lch, Rgb, Yuv,
};
use crate::config::ColorConfig;

/// 画像から指定色のクロマキーマスクを生成
///
/// # Arguments
/// * `image` - 入力画像
/// * `target_color` - クロマキー対象色
/// * `tolerance` - 色の許容範囲 (0.0 - 1.0)
/// * `color_space` - 使用する色空間
///
/// # Returns
/// グレースケールマスク（白=クロマキー対象、黒=保持）
pub fn create_chroma_mask(
    image: &RgbaImage,
    target_color: &Rgb,
    tolerance: f32,
    color_space: ColorSpace,
) -> GrayImage {
    let (width, height) = image.dimensions();

    // RGB事前フィルタリングの閾値（改善案B）
    let rgb_threshold_squared = ((tolerance * 2.0 * 441.67) as u32).pow(2);

    // 入力バッファを取得（改善案D）
    let src = image.as_raw();

    // 色空間に応じた処理
    match color_space {
        ColorSpace::Hsv => {
            let lut = HsvLut::new();
            let target_hsv = rgb_to_hsv(target_color);

            #[cfg(feature = "parallel")]
            let mask_data: Vec<u8> = (0..height)
                .into_par_iter()
                .flat_map(|y| {
                    process_mask_row_hsv(
                        y, width, src, &lut, target_color, &target_hsv, tolerance, rgb_threshold_squared
                    )
                })
                .collect();

            #[cfg(not(feature = "parallel"))]
            let mask_data: Vec<u8> = (0..height)
                .flat_map(|y| {
                    process_mask_row_hsv(
                        y, width, src, &lut, target_color, &target_hsv, tolerance, rgb_threshold_squared
                    )
                })
                .collect();

            GrayImage::from_raw(width, height, mask_data).expect("Failed to create mask image")
        }
        ColorSpace::Lab => {
            let target_lab = rgb_to_lab(target_color);

            #[cfg(feature = "parallel")]
            let mask_data: Vec<u8> = (0..height)
                .into_par_iter()
                .flat_map(|y| {
                    process_mask_row_lab(y, width, src, target_color, &target_lab, tolerance, rgb_threshold_squared)
                })
                .collect();

            #[cfg(not(feature = "parallel"))]
            let mask_data: Vec<u8> = (0..height)
                .flat_map(|y| {
                    process_mask_row_lab(y, width, src, target_color, &target_lab, tolerance, rgb_threshold_squared)
                })
                .collect();

            GrayImage::from_raw(width, height, mask_data).expect("Failed to create mask image")
        }
        ColorSpace::Lch => {
            let target_lch = rgb_to_lch(target_color);

            #[cfg(feature = "parallel")]
            let mask_data: Vec<u8> = (0..height)
                .into_par_iter()
                .flat_map(|y| {
                    process_mask_row_lch(y, width, src, target_color, &target_lch, tolerance, rgb_threshold_squared)
                })
                .collect();

            #[cfg(not(feature = "parallel"))]
            let mask_data: Vec<u8> = (0..height)
                .flat_map(|y| {
                    process_mask_row_lch(y, width, src, target_color, &target_lch, tolerance, rgb_threshold_squared)
                })
                .collect();

            GrayImage::from_raw(width, height, mask_data).expect("Failed to create mask image")
        }
        ColorSpace::Yuv => {
            let target_yuv = rgb_to_yuv(target_color);

            #[cfg(feature = "parallel")]
            let mask_data: Vec<u8> = (0..height)
                .into_par_iter()
                .flat_map(|y| {
                    process_mask_row_yuv(y, width, src, target_color, &target_yuv, tolerance, rgb_threshold_squared)
                })
                .collect();

            #[cfg(not(feature = "parallel"))]
            let mask_data: Vec<u8> = (0..height)
                .flat_map(|y| {
                    process_mask_row_yuv(y, width, src, target_color, &target_yuv, tolerance, rgb_threshold_squared)
                })
                .collect();

            GrayImage::from_raw(width, height, mask_data).expect("Failed to create mask image")
        }
    }
}

/// 複数の色を同時に検出してマスクを生成
///
/// # Arguments
/// * `image` - 入力画像
/// * `color_configs` - 色設定のリスト（各色と許容範囲）
/// * `color_space` - 使用する色空間
///
/// # Returns
/// グレースケールマスク（白=クロマキー対象、黒=保持）
/// 複数のマスクをOR演算で統合
pub fn create_multi_chroma_mask(
    image: &RgbaImage,
    color_configs: &[ColorConfig],
    color_space: ColorSpace,
) -> GrayImage {
    if color_configs.is_empty() {
        let (width, height) = image.dimensions();
        return GrayImage::new(width, height);
    }

    let masks: Vec<GrayImage> = color_configs
        .iter()
        .map(|config| create_chroma_mask(image, &config.color, config.tolerance, color_space))
        .collect();

    combine_masks(&masks)
}

/// 複数のマスクをOR演算で統合
fn combine_masks(masks: &[GrayImage]) -> GrayImage {
    if masks.is_empty() {
        // 空の場合は最初のマスクのサイズで空のマスクを返す
        if let Some(first) = masks.first() {
            return GrayImage::new(first.width(), first.height());
        }
        return GrayImage::new(1, 1);
    }

    let (width, height) = masks[0].dimensions();
    let mut result = vec![0u8; (width * height) as usize];

    // 各マスクのピクセルをOR演算
    for mask in masks {
        let mask_raw = mask.as_raw();
        for (i, &val) in mask_raw.iter().enumerate() {
            if i < result.len() {
                result[i] = result[i].max(val);
            }
        }
    }

    GrayImage::from_raw(width, height, result).expect("Failed to create combined mask")
}

/// 1行分のマスクデータを処理（HSV）
fn process_mask_row_hsv(
    y: u32,
    width: u32,
    src: &[u8],
    lut: &HsvLut,
    target_color: &Rgb,
    target_hsv: &Hsv,
    tolerance: f32,
    rgb_threshold_squared: u32,
) -> Vec<u8> {
    let mut row = Vec::with_capacity(width as usize);
    for x in 0..width {
        let idx = ((y * width + x) * 4) as usize;
        let r = src[idx];
        let g = src[idx + 1];
        let b = src[idx + 2];

        let pixel_rgb = Rgb::new(r, g, b);

        let is_chroma = if rgb_distance_squared(&pixel_rgb, target_color) > rgb_threshold_squared {
            false
        } else {
            let pixel_hsv = lut.get(&pixel_rgb);
            pixel_hsv.is_within_tolerance(target_hsv, tolerance)
        };

        row.push(if is_chroma { 255 } else { 0 });
    }
    row
}

/// 1行分のマスクデータを処理（LAB）
fn process_mask_row_lab(
    y: u32,
    width: u32,
    src: &[u8],
    target_color: &Rgb,
    target_lab: &Lab,
    tolerance: f32,
    rgb_threshold_squared: u32,
) -> Vec<u8> {
    let mut row = Vec::with_capacity(width as usize);
    for x in 0..width {
        let idx = ((y * width + x) * 4) as usize;
        let r = src[idx];
        let g = src[idx + 1];
        let b = src[idx + 2];

        let pixel_rgb = Rgb::new(r, g, b);

        let is_chroma = if rgb_distance_squared(&pixel_rgb, target_color) > rgb_threshold_squared {
            false
        } else {
            let pixel_lab = rgb_to_lab(&pixel_rgb);
            pixel_lab.is_within_tolerance(target_lab, tolerance)
        };

        row.push(if is_chroma { 255 } else { 0 });
    }
    row
}

/// 1行分のマスクデータを処理（LCH）
fn process_mask_row_lch(
    y: u32,
    width: u32,
    src: &[u8],
    target_color: &Rgb,
    target_lch: &Lch,
    tolerance: f32,
    rgb_threshold_squared: u32,
) -> Vec<u8> {
    let mut row = Vec::with_capacity(width as usize);
    for x in 0..width {
        let idx = ((y * width + x) * 4) as usize;
        let r = src[idx];
        let g = src[idx + 1];
        let b = src[idx + 2];

        let pixel_rgb = Rgb::new(r, g, b);

        let is_chroma = if rgb_distance_squared(&pixel_rgb, target_color) > rgb_threshold_squared {
            false
        } else {
            let pixel_lch = rgb_to_lch(&pixel_rgb);
            pixel_lch.is_within_tolerance(target_lch, tolerance)
        };

        row.push(if is_chroma { 255 } else { 0 });
    }
    row
}

/// 1行分のマスクデータを処理（YUV）
fn process_mask_row_yuv(
    y: u32,
    width: u32,
    src: &[u8],
    target_color: &Rgb,
    target_yuv: &Yuv,
    tolerance: f32,
    rgb_threshold_squared: u32,
) -> Vec<u8> {
    let mut row = Vec::with_capacity(width as usize);
    for x in 0..width {
        let idx = ((y * width + x) * 4) as usize;
        let r = src[idx];
        let g = src[idx + 1];
        let b = src[idx + 2];

        let pixel_rgb = Rgb::new(r, g, b);

        let is_chroma = if rgb_distance_squared(&pixel_rgb, target_color) > rgb_threshold_squared {
            false
        } else {
            let pixel_yuv = rgb_to_yuv(&pixel_rgb);
            pixel_yuv.is_within_tolerance(target_yuv, tolerance)
        };

        row.push(if is_chroma { 255 } else { 0 });
    }
    row
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
        let mask_low = create_chroma_mask(&image, &target, 0.1, ColorSpace::Hsv);
        assert_eq!(mask_low.get_pixel(0, 0).0[0], 255); // 完全な緑は検出

        // 高い許容範囲
        let mask_high = create_chroma_mask(&image, &target, 0.5, ColorSpace::Hsv);
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
        let mask = create_chroma_mask(&image, &target, 0.3, ColorSpace::Hsv);

        // 左半分は白、右半分は黒
        assert_eq!(mask.get_pixel(0, 0).0[0], 255);
        assert_eq!(mask.get_pixel(999, 0).0[0], 0);
    }
}
