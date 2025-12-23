//! フェザリング（ガウシアンブラー）（最適化版）
//!
//! 改善案A: Rayon並列処理（parallel feature有効時）
//! 改善案D: バッファ直接操作

use image::GrayImage;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// ガウシアンブラーでアルファチャンネルをフェザリング
/// エッジを滑らかにしてギザギザを防ぐ
///
/// # Arguments
/// * `alpha` - アルファチャンネル画像
/// * `amount` - ブラー量（カーネルサイズ = amount * 2 + 1）
pub fn feather_alpha(alpha: &GrayImage, amount: u32) -> GrayImage {
    if amount == 0 {
        return alpha.clone();
    }

    // カーネルサイズは奇数にする
    let kernel_size = amount * 2 + 1;
    let sigma = amount as f32 / 2.0;

    gaussian_blur(alpha, kernel_size, sigma)
}

/// ガウシアンブラー（分離カーネル + 並列処理）
///
/// 改善案A: 行/列ごとの並列処理
/// 改善案D: バッファ直接操作
fn gaussian_blur(image: &GrayImage, kernel_size: u32, sigma: f32) -> GrayImage {
    // ガウシアンカーネルを生成
    let kernel = generate_gaussian_kernel(kernel_size, sigma);
    let radius = (kernel_size / 2) as i32;

    let (width, height) = image.dimensions();
    let w = width as usize;
    let h = height as usize;
    let src = image.as_raw();

    // 水平方向のブラー
    #[cfg(feature = "parallel")]
    let horizontal: Vec<u8> = (0..h)
        .into_par_iter()
        .flat_map(|y| blur_row_horizontal(y, w, src, &kernel, radius))
        .collect();

    #[cfg(not(feature = "parallel"))]
    let horizontal: Vec<u8> = (0..h)
        .flat_map(|y| blur_row_horizontal(y, w, src, &kernel, radius))
        .collect();

    // 垂直方向のブラー
    #[cfg(feature = "parallel")]
    let result: Vec<u8> = (0..h)
        .into_par_iter()
        .flat_map(|y| blur_row_vertical(y, w, h, &horizontal, &kernel, radius))
        .collect();

    #[cfg(not(feature = "parallel"))]
    let result: Vec<u8> = (0..h)
        .flat_map(|y| blur_row_vertical(y, w, h, &horizontal, &kernel, radius))
        .collect();

    GrayImage::from_raw(width, height, result).expect("Failed to create image")
}

/// 水平方向のブラー（1行）
fn blur_row_horizontal(y: usize, w: usize, src: &[u8], kernel: &[f32], radius: i32) -> Vec<u8> {
    let mut row = Vec::with_capacity(w);
    for x in 0..w {
        let mut sum = 0.0f32;
        let mut weight_sum = 0.0f32;

        for i in -radius..=radius {
            let nx = x as i32 + i;
            if nx >= 0 && nx < w as i32 {
                let val = src[y * w + nx as usize] as f32;
                let weight = kernel[(i + radius) as usize];
                sum += val * weight;
                weight_sum += weight;
            }
        }

        row.push((sum / weight_sum).round() as u8);
    }
    row
}

/// 垂直方向のブラー（1行）
fn blur_row_vertical(y: usize, w: usize, h: usize, src: &[u8], kernel: &[f32], radius: i32) -> Vec<u8> {
    let mut row = Vec::with_capacity(w);
    for x in 0..w {
        let mut sum = 0.0f32;
        let mut weight_sum = 0.0f32;

        for i in -radius..=radius {
            let ny = y as i32 + i;
            if ny >= 0 && ny < h as i32 {
                let val = src[ny as usize * w + x] as f32;
                let weight = kernel[(i + radius) as usize];
                sum += val * weight;
                weight_sum += weight;
            }
        }

        row.push((sum / weight_sum).round() as u8);
    }
    row
}

/// 1次元ガウシアンカーネルを生成
fn generate_gaussian_kernel(size: u32, sigma: f32) -> Vec<f32> {
    let mut kernel = Vec::with_capacity(size as usize);
    let center = (size / 2) as f32;

    for i in 0..size {
        let x = i as f32 - center;
        let value = (-x * x / (2.0 * sigma * sigma)).exp();
        kernel.push(value);
    }

    // 正規化
    let sum: f32 = kernel.iter().sum();
    for val in kernel.iter_mut() {
        *val /= sum;
    }

    kernel
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feather_zero_amount() {
        let mut alpha = GrayImage::new(3, 3);
        alpha.put_pixel(1, 1, image::Luma([255]));

        let feathered = feather_alpha(&alpha, 0);

        // 変更なし
        assert_eq!(feathered.get_pixel(1, 1).0[0], 255);
        assert_eq!(feathered.get_pixel(0, 0).0[0], 0);
    }

    #[test]
    fn test_feather_smooths_edge() {
        let mut alpha = GrayImage::new(5, 5);
        // 左半分を白に
        for y in 0..5 {
            for x in 0..3 {
                alpha.put_pixel(x, y, image::Luma([255]));
            }
        }

        let feathered = feather_alpha(&alpha, 1);

        // 境界がぼかされる（中間値になる）
        let edge_val = feathered.get_pixel(2, 2).0[0];
        assert!(edge_val > 0 && edge_val < 255);
    }

    #[test]
    fn test_gaussian_kernel() {
        let kernel = generate_gaussian_kernel(5, 1.0);
        assert_eq!(kernel.len(), 5);

        // 合計が1.0になる
        let sum: f32 = kernel.iter().sum();
        assert!((sum - 1.0).abs() < 0.001);

        // 中心が最大
        assert!(kernel[2] > kernel[1]);
        assert!(kernel[2] > kernel[3]);
    }

    #[test]
    fn test_large_image() {
        // 並列処理のテスト用大きな画像
        let alpha = GrayImage::from_fn(500, 500, |x, _| {
            if x < 250 {
                image::Luma([255])
            } else {
                image::Luma([0])
            }
        });

        let feathered = feather_alpha(&alpha, 5);
        // 境界付近がぼかされている
        let edge = feathered.get_pixel(250, 250).0[0];
        assert!(edge > 0 && edge < 255);
    }
}
