//! 影の処理
//!
//! 背景の影を検出して除去する機能

use image::{GrayImage, RgbaImage};

#[cfg(feature = "parallel")]
use rayon::prelude::*;


/// 影を検出してマスクから除去
///
/// # Arguments
/// * `mask` - クロマキーマスク
/// * `image` - 元画像（明度分析用）
/// * `threshold` - 影検出の閾値（0.0 - 1.0）
/// * `strength` - 影除去の強度（0.0 - 1.0）
///
/// # Returns
/// 影が除去されたマスク
pub fn remove_shadows(
    mask: &GrayImage,
    image: &RgbaImage,
    threshold: f32,
    strength: f32,
) -> GrayImage {
    if strength <= 0.0 {
        return mask.clone();
    }

    let (width, height) = mask.dimensions();
    let w = width as usize;
    #[allow(unused_variables)] // parallel feature無効時のみ使用
    let h = height as usize;
    let mask_raw = mask.as_raw();
    let _image_raw = image.as_raw();

    // 明度（Luminance）を計算
    let luminance = calculate_luminance(image);

    // 影領域を検出
    let shadow_mask = detect_shadows(&luminance, threshold);

    // マスクから影領域を除去
    let mut result = vec![0u8; (width * height) as usize];

    #[cfg(feature = "parallel")]
    {
        result
            .par_chunks_mut(w)
            .enumerate()
            .for_each(|(y, row)| {
                remove_shadows_row(
                    row,
                    y,
                    w,
                    mask_raw,
                    &shadow_mask,
                    strength,
                );
            });
    }

    #[cfg(not(feature = "parallel"))]
    {
        for y in 0..h {
            let row = &mut result[y * w..(y + 1) * w];
            remove_shadows_row(row, y, w, mask_raw, &shadow_mask, strength);
        }
    }

    GrayImage::from_raw(width, height, result).expect("Failed to create shadow-removed mask")
}

/// 画像の明度を計算
fn calculate_luminance(image: &RgbaImage) -> GrayImage {
    let (width, height) = image.dimensions();
    let w = width as usize;
    #[allow(unused_variables)] // parallel feature無効時のみ使用
    let h = height as usize;
    let src = image.as_raw();
    let mut result = vec![0u8; (width * height) as usize];

    #[cfg(feature = "parallel")]
    {
        result
            .par_chunks_mut(w)
            .enumerate()
            .for_each(|(y, row)| {
                for x in 0..w {
                    let idx = (y * w + x) * 4;
                    if idx + 2 < src.len() {
                        // ITU-R BT.601標準の重み
                        let r = src[idx] as f32;
                        let g = src[idx + 1] as f32;
                        let b = src[idx + 2] as f32;
                        let lum = 0.299 * r + 0.587 * g + 0.114 * b;
                        row[x] = lum.clamp(0.0, 255.0) as u8;
                    }
                }
            });
    }

    #[cfg(not(feature = "parallel"))]
    {
        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) * 4;
                if idx + 2 < src.len() {
                    let r = src[idx] as f32;
                    let g = src[idx + 1] as f32;
                    let b = src[idx + 2] as f32;
                    let lum = 0.299 * r + 0.587 * g + 0.114 * b;
                    result[y * w + x] = lum.clamp(0.0, 255.0) as u8;
                }
            }
        }
    }

    GrayImage::from_raw(width, height, result).expect("Failed to create luminance image")
}

/// 影領域を検出
fn detect_shadows(luminance: &GrayImage, threshold: f32) -> GrayImage {
    let (width, height) = luminance.dimensions();
    let _w = width as usize;
    let _h = height as usize;
    let src = luminance.as_raw();

    // 平均明度を計算
    let avg_lum: f32 = src.iter().map(|&v| v as f32).sum::<f32>() / src.len() as f32;
    let shadow_threshold = avg_lum * threshold;

    let mut result = vec![0u8; (width * height) as usize];

    for (i, &lum) in src.iter().enumerate() {
        // 平均より暗い領域を影として検出
        if (lum as f32) < shadow_threshold {
            result[i] = 255; // 影領域
        } else {
            result[i] = 0; // 非影領域
        }
    }

    GrayImage::from_raw(width, height, result).expect("Failed to create shadow mask")
}

/// 1行の影除去処理
fn remove_shadows_row(
    row: &mut [u8],
    y: usize,
    w: usize,
    mask_raw: &[u8],
    shadow_mask: &GrayImage,
    strength: f32,
) {
    let shadow_raw = shadow_mask.as_raw();
    for x in 0..w {
        let idx = y * w + x;
        let mask_val = mask_raw[idx];
        let shadow_val = shadow_raw[idx];

        // 影領域でマスクが有効な場合、強度に応じて除去
        if shadow_val > 0 && mask_val > 0 {
            let removal = (mask_val as f32 * strength * (shadow_val as f32 / 255.0)) as u8;
            row[x] = mask_val.saturating_sub(removal);
        } else {
            row[x] = mask_val;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_remove_shadows_zero_strength() {
        let mask = GrayImage::from_fn(10, 10, |_, _| image::Luma([255]));
        let image = RgbaImage::from_fn(10, 10, |_, _| Rgba([128, 128, 128, 255]));
        let result = remove_shadows(&mask, &image, 0.3, 0.0);
        // 強度0の場合は変化なし
        assert_eq!(result.get_pixel(5, 5).0[0], 255);
    }

    #[test]
    fn test_calculate_luminance() {
        let image = RgbaImage::from_fn(1, 1, |_, _| Rgba([255, 128, 0, 255]));
        let lum = calculate_luminance(&image);
        let val = lum.get_pixel(0, 0).0[0];
        // 明度は0-255の範囲内
        assert!(val > 0 && val < 255);
    }
}

