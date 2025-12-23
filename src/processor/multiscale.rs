//! マルチスケール処理
//!
//! 複数の解像度で処理して統合することで、細部と全体のバランスを取る

use image::{GrayImage, RgbaImage};
use image::imageops::FilterType;

use crate::color::{ColorSpace, Rgb};
use crate::processor::mask::create_chroma_mask;

/// マルチスケール処理でマスクを生成
///
/// # Arguments
/// * `image` - 入力画像
/// * `target_color` - クロマキー対象色
/// * `tolerance` - 色の許容範囲
/// * `levels` - スケールレベル数
/// * `scale_factor` - スケール係数（各レベルでの縮小率）
/// * `color_space` - 使用する色空間
///
/// # Returns
/// 統合されたマスク
pub fn create_multiscale_mask(
    image: &RgbaImage,
    target_color: &Rgb,
    tolerance: f32,
    levels: u32,
    scale_factor: f32,
    color_space: ColorSpace,
) -> GrayImage {
    if levels == 0 {
        return create_chroma_mask(image, target_color, tolerance, color_space);
    }

    let (width, height) = image.dimensions();
    let mut masks = Vec::new();

    // 各スケールレベルでマスクを生成
    for level in 0..levels {
        let scale = scale_factor.powi(level as i32);
        let scaled_width = (width as f32 * scale) as u32;
        let scaled_height = (height as f32 * scale) as u32;

        if scaled_width == 0 || scaled_height == 0 {
            continue;
        }

        // 画像をリサイズ
        let scaled_image = image::imageops::resize(
            image,
            scaled_width,
            scaled_height,
            FilterType::Triangle,
        );

        // マスクを生成
        let mask = create_chroma_mask(&scaled_image, target_color, tolerance, color_space);

        // 元のサイズにアップスケール
        let upscaled_mask = image::imageops::resize(
            &mask,
            width,
            height,
            FilterType::Triangle,
        );

        masks.push((level, upscaled_mask));
    }

    // マスクを統合（重み付き平均）
    combine_multiscale_masks(&masks, width, height)
}

/// 複数のスケールマスクを統合
fn combine_multiscale_masks(masks: &[(u32, GrayImage)], width: u32, height: u32) -> GrayImage {
    if masks.is_empty() {
        return GrayImage::new(width, height);
    }

    let mut result = vec![0.0f32; (width * height) as usize];
    let mut total_weight = 0.0f32;

    // 各マスクを重み付きで加算
    // 高解像度（レベル0）ほど重みを大きくする
    for (level, mask) in masks {
        let weight = 1.0 / ((level + 1) as f32);
        let mask_raw = mask.as_raw();

        for (i, &val) in mask_raw.iter().enumerate() {
            if i < result.len() {
                result[i] += val as f32 * weight;
            }
        }
        total_weight += weight;
    }

    // 正規化
    if total_weight > 0.0 {
        for val in result.iter_mut() {
            *val /= total_weight;
        }
    }

    // u8に変換
    let result_u8: Vec<u8> = result
        .iter()
        .map(|&v| v.round().clamp(0.0, 255.0) as u8)
        .collect();

    GrayImage::from_raw(width, height, result_u8).expect("Failed to create multiscale mask")
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_multiscale_zero_levels() {
        let image = RgbaImage::from_fn(10, 10, |_, _| Rgba([0, 255, 0, 255]));
        let target = Rgb::new(0, 255, 0);
        let result = create_multiscale_mask(&image, &target, 0.3, 0, 0.5, ColorSpace::Hsv);
        // レベル0の場合は通常のマスク生成と同じ
        assert_eq!(result.width(), 10);
        assert_eq!(result.height(), 10);
    }

    #[test]
    fn test_multiscale_basic() {
        let image = RgbaImage::from_fn(100, 100, |_, _| Rgba([0, 255, 0, 255]));
        let target = Rgb::new(0, 255, 0);
        let result = create_multiscale_mask(&image, &target, 0.3, 3, 0.5, ColorSpace::Hsv);
        // マスクが生成される
        assert_eq!(result.width(), 100);
        assert_eq!(result.height(), 100);
    }
}

