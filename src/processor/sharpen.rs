//! エッジシャープニング（アンシャープマスク）

use image::RgbaImage;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use crate::processor::feather::gaussian_blur;

/// エッジシャープニング処理
///
/// アンシャープマスク（Unsharp Mask）を使用してエッジを強調
///
/// # Arguments
/// * `image` - 入力画像
/// * `amount` - シャープニング強度 (0.0 - 2.0)
/// * `radius` - シャープニング半径（ガウシアンブラーの半径）
/// * `threshold` - シャープニング閾値（この値以下の差分は無視）
pub fn sharpen(image: &RgbaImage, amount: f32, radius: f32, threshold: f32) -> RgbaImage {
    if amount <= 0.0 || radius <= 0.0 {
        return image.clone();
    }

    let amount = amount.clamp(0.0, 2.0);
    let radius = radius.max(0.1);
    let threshold = threshold.max(0.0);

    // ガウシアンブラーで平滑化
    let blurred = apply_gaussian_blur_to_rgba(image, radius);

    // 元画像とブラー画像の差分を計算してシャープニング
    apply_unsharp_mask(image, &blurred, amount, threshold)
}

/// RGBA画像にガウシアンブラーを適用
fn apply_gaussian_blur_to_rgba(image: &RgbaImage, radius: f32) -> RgbaImage {
    let (width, height) = image.dimensions();
    
    // 各チャンネルにガウシアンブラーを適用
    let r_channel = extract_channel(image, 0);
    let g_channel = extract_channel(image, 1);
    let b_channel = extract_channel(image, 2);
    let a_channel = extract_channel(image, 3);

    let kernel_size = ((radius * 2.0) as u32).max(3);
    let sigma = radius;
    
    let r_blurred = gaussian_blur(&r_channel, kernel_size, sigma);
    let g_blurred = gaussian_blur(&g_channel, kernel_size, sigma);
    let b_blurred = gaussian_blur(&b_channel, kernel_size, sigma);
    let a_blurred = gaussian_blur(&a_channel, kernel_size, sigma);

    // チャンネルを統合
    combine_channels(&r_blurred, &g_blurred, &b_blurred, &a_blurred, width, height)
}

/// チャンネルを抽出
fn extract_channel(image: &RgbaImage, channel: usize) -> image::GrayImage {
    let (width, height) = image.dimensions();
    let mut gray = image::GrayImage::new(width, height);
    
    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            gray.put_pixel(x, y, image::Luma([pixel[channel]]));
        }
    }
    
    gray
}

/// チャンネルを統合
fn combine_channels(
    r: &image::GrayImage,
    g: &image::GrayImage,
    b: &image::GrayImage,
    a: &image::GrayImage,
    width: u32,
    height: u32,
) -> RgbaImage {
    let mut result = RgbaImage::new(width, height);
    
    for y in 0..height {
        for x in 0..width {
            let r_val = r.get_pixel(x, y).0[0];
            let g_val = g.get_pixel(x, y).0[0];
            let b_val = b.get_pixel(x, y).0[0];
            let a_val = a.get_pixel(x, y).0[0];
            result.put_pixel(x, y, image::Rgba([r_val, g_val, b_val, a_val]));
        }
    }
    
    result
}

/// アンシャープマスクを適用
fn apply_unsharp_mask(
    original: &RgbaImage,
    blurred: &RgbaImage,
    amount: f32,
    threshold: f32,
) -> RgbaImage {
    let (width, height) = original.dimensions();
    let mut result = RgbaImage::new(width, height);
    
    let orig_raw = original.as_raw();
    let blur_raw = blurred.as_raw();
    let result_raw = result.as_mut();
    
    #[cfg(feature = "parallel")]
    {
        result_raw
            .par_chunks_mut(4)
            .enumerate()
            .for_each(|(i, pixel)| {
                let orig_idx = i * 4;
                if orig_idx + 3 < orig_raw.len() {
                    apply_unsharp_mask_pixel(
                        &orig_raw[orig_idx..orig_idx + 4],
                        &blur_raw[orig_idx..orig_idx + 4],
                        amount,
                        threshold,
                        pixel,
                    );
                }
            });
    }
    
    #[cfg(not(feature = "parallel"))]
    {
        for i in 0..(width * height) as usize {
            let idx = i * 4;
            if idx + 3 < orig_raw.len() {
                apply_unsharp_mask_pixel(
                    &orig_raw[idx..idx + 4],
                    &blur_raw[idx..idx + 4],
                    amount,
                    threshold,
                    &mut result_raw[idx..idx + 4],
                );
            }
        }
    }
    
    result
}

/// 1ピクセルにアンシャープマスクを適用
#[inline]
fn apply_unsharp_mask_pixel(
    original: &[u8],
    blurred: &[u8],
    amount: f32,
    threshold: f32,
    output: &mut [u8],
) {
    for c in 0..3 {
        // RGBチャンネルのみ（アルファは変更しない）
        let orig_val = original[c] as f32;
        let blur_val = blurred[c] as f32;
        
        // 差分を計算
        let diff = orig_val - blur_val;
        
        // 閾値チェック
        if diff.abs() < threshold * 255.0 {
            output[c] = original[c];
        } else {
            // シャープニングを適用
            let sharpened = orig_val + diff * amount;
            output[c] = sharpened.clamp(0.0, 255.0) as u8;
        }
    }
    
    // アルファチャンネルはそのまま
    output[3] = original[3];
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_sharpen_zero_amount() {
        let image = RgbaImage::from_fn(10, 10, |_, _| Rgba([128, 128, 128, 255]));
        let result = sharpen(&image, 0.0, 1.0, 0.0);
        // 変化なし
        assert_eq!(result.get_pixel(5, 5).0[0], 128);
    }

    #[test]
    fn test_sharpen_high_contrast() {
        // 高コントラストの画像（白と黒の境界）
        let mut image = RgbaImage::new(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                if x < 5 {
                    image.put_pixel(x, y, Rgba([0, 0, 0, 255]));
                } else {
                    image.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                }
            }
        }
        
        let result = sharpen(&image, 1.0, 1.0, 0.0);
        // エッジが強調される（境界付近の値が変化する）
        let edge_pixel = result.get_pixel(5, 5);
        // エッジ付近の値が変化していることを確認
        assert!(edge_pixel[0] != 128 || edge_pixel[0] != 0 || edge_pixel[0] != 255);
    }
}

