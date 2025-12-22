//! フェザリング（ガウシアンブラー）

use image::GrayImage;

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

/// ガウシアンブラー（高品質）
fn gaussian_blur(image: &GrayImage, kernel_size: u32, sigma: f32) -> GrayImage {
    // ガウシアンカーネルを生成
    let kernel = generate_gaussian_kernel(kernel_size, sigma);
    let radius = (kernel_size / 2) as i32;

    let (width, height) = image.dimensions();

    // 水平方向のブラー
    let mut horizontal = GrayImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let mut sum = 0.0f32;
            let mut weight_sum = 0.0f32;

            for i in -radius..=radius {
                let nx = x as i32 + i;
                if nx >= 0 && nx < width as i32 {
                    let val = image.get_pixel(nx as u32, y).0[0] as f32;
                    let weight = kernel[(i + radius) as usize];
                    sum += val * weight;
                    weight_sum += weight;
                }
            }

            let result = (sum / weight_sum).round() as u8;
            horizontal.put_pixel(x, y, image::Luma([result]));
        }
    }

    // 垂直方向のブラー
    let mut result = GrayImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let mut sum = 0.0f32;
            let mut weight_sum = 0.0f32;

            for i in -radius..=radius {
                let ny = y as i32 + i;
                if ny >= 0 && ny < height as i32 {
                    let val = horizontal.get_pixel(x, ny as u32).0[0] as f32;
                    let weight = kernel[(i + radius) as usize];
                    sum += val * weight;
                    weight_sum += weight;
                }
            }

            let result_val = (sum / weight_sum).round() as u8;
            result.put_pixel(x, y, image::Luma([result_val]));
        }
    }

    result
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
}

