//! アルファチャンネル処理（最適化版）
//!
//! 改善案A: Rayon並列処理（parallel feature有効時）
//! 改善案D: バッファ直接操作
//! 改善案E: SIMD最適化（非WASM環境のみ）

use image::{GrayImage, RgbaImage};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use super::simd;

/// マスクからアルファチャンネルを生成
///
/// マスク（白=クロマキー対象）を反転してアルファチャンネルに変換
///
/// 改善案E: SIMD最適化で高速化（非WASM環境）
pub fn create_alpha_from_mask(mask: &GrayImage) -> GrayImage {
    let (width, height) = mask.dimensions();
    let src = mask.as_raw();
    let mut dst = vec![0u8; src.len()];

    #[cfg(not(target_arch = "wasm32"))]
    {
        // SIMDで反転（改善案E）
        simd::invert_mask_simd(src, &mut dst);
    }

    #[cfg(target_arch = "wasm32")]
    {
        // スカラー処理（WASM）
        for (d, s) in dst.iter_mut().zip(src.iter()) {
            *d = 255 - s;
        }
    }

    GrayImage::from_raw(width, height, dst).expect("Failed to create alpha channel")
}

/// アルファチャンネルを画像に適用
///
/// 改善案A: 並列処理
/// 改善案D: バッファ直接操作
/// 改善案E: SIMD最適化（非WASM環境）
pub fn apply_alpha(image: &RgbaImage, alpha: &GrayImage) -> RgbaImage {
    let mut result = image.clone();

    #[cfg(not(target_arch = "wasm32"))]
    {
        // SIMDでアルファ適用（改善案E）
        simd::apply_alpha_simd(result.as_mut(), alpha.as_raw());
    }

    #[cfg(target_arch = "wasm32")]
    {
        // スカラー処理（WASM）
        apply_alpha_scalar(result.as_mut(), alpha.as_raw());
    }

    result
}

/// アルファチャンネルを画像にインプレース適用
///
/// 改善案G: インプレース処理
pub fn apply_alpha_inplace(image: &mut RgbaImage, alpha: &GrayImage) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        // SIMDでアルファ適用（改善案E）
        simd::apply_alpha_simd(image.as_mut(), alpha.as_raw());
    }

    #[cfg(target_arch = "wasm32")]
    {
        // スカラー処理（WASM）
        apply_alpha_scalar(image.as_mut(), alpha.as_raw());
    }
}

/// スカラー処理: アルファ適用
#[cfg(target_arch = "wasm32")]
fn apply_alpha_scalar(rgba: &mut [u8], alpha: &[u8]) {
    for (i, &a) in alpha.iter().enumerate() {
        if i * 4 + 3 < rgba.len() {
            rgba[i * 4 + 3] = a;
        }
    }
}

/// マスクからアルファチャンネルを生成（並列版）
///
/// 改善案A: Rayon並列処理
#[cfg(feature = "parallel")]
pub fn create_alpha_from_mask_parallel(mask: &GrayImage) -> GrayImage {
    let (width, height) = mask.dimensions();
    let w = width as usize;

    let result: Vec<u8> = mask
        .as_raw()
        .par_chunks(w)
        .flat_map(|row| row.iter().map(|&m| 255 - m).collect::<Vec<u8>>())
        .collect();

    GrayImage::from_raw(width, height, result).expect("Failed to create alpha channel")
}

/// マスクからアルファチャンネルを生成（シーケンシャル版）
#[cfg(not(feature = "parallel"))]
#[allow(dead_code)]
pub fn create_alpha_from_mask_parallel(mask: &GrayImage) -> GrayImage {
    // parallel無効時は通常の関数にフォールバック
    create_alpha_from_mask(mask)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_create_alpha_from_mask() {
        let mut mask = GrayImage::new(2, 1);
        mask.put_pixel(0, 0, image::Luma([255])); // クロマキー対象
        mask.put_pixel(1, 0, image::Luma([0])); // 保持

        let alpha = create_alpha_from_mask(&mask);

        // 反転される
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

        let result = apply_alpha(&image, &alpha);

        assert_eq!(result.get_pixel(0, 0).0[3], 128);
        assert_eq!(result.get_pixel(1, 0).0[3], 255);
    }

    #[test]
    fn test_apply_alpha_inplace() {
        let mut image = RgbaImage::new(2, 1);
        image.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        image.put_pixel(1, 0, Rgba([0, 255, 0, 255]));

        let mut alpha = GrayImage::new(2, 1);
        alpha.put_pixel(0, 0, image::Luma([128]));
        alpha.put_pixel(1, 0, image::Luma([255]));

        apply_alpha_inplace(&mut image, &alpha);

        assert_eq!(image.get_pixel(0, 0).0[3], 128);
        assert_eq!(image.get_pixel(1, 0).0[3], 255);
    }

    #[test]
    fn test_large_image() {
        // 並列処理とSIMDのテスト用大きな画像
        let image = RgbaImage::from_fn(1000, 1000, |_, _| Rgba([255, 0, 0, 255]));

        let alpha = GrayImage::from_fn(1000, 1000, |x, _| {
            if x < 500 {
                image::Luma([0])
            } else {
                image::Luma([255])
            }
        });

        let result = apply_alpha(&image, &alpha);

        // 左半分は透明、右半分は不透明
        assert_eq!(result.get_pixel(0, 0).0[3], 0);
        assert_eq!(result.get_pixel(999, 0).0[3], 255);
    }
}
