//! バイラテラルフィルタ
//!
//! エッジを保持しながらノイズを除去するフィルタ
//! 空間的重みと色の重みを組み合わせて平滑化

use image::GrayImage;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// バイラテラルフィルタをアルファチャンネルに適用
///
/// # Arguments
/// * `alpha` - アルファチャンネル画像
/// * `spatial_sigma` - 空間的重みの標準偏差
/// * `color_sigma` - 色の重みの標準偏差
/// * `radius` - カーネル半径
pub fn bilateral_filter_alpha(
    alpha: &GrayImage,
    spatial_sigma: f32,
    color_sigma: f32,
    radius: u32,
) -> GrayImage {
    if radius == 0 {
        return alpha.clone();
    }

    let (width, height) = alpha.dimensions();
    let w = width as usize;
    let h = height as usize;
    let src = alpha.as_raw();

    // 空間的重みの事前計算
    let spatial_weights = generate_spatial_weights(radius, spatial_sigma);

    // 色の重みの正規化係数
    let color_factor = -0.5 / (color_sigma * color_sigma);

    let mut result = vec![0u8; (width * height) as usize];

    #[cfg(feature = "parallel")]
    {
        result
            .par_chunks_mut(w)
            .enumerate()
            .for_each(|(y, row)| {
                bilateral_filter_row(
                    row,
                    y,
                    w,
                    h,
                    src,
                    &spatial_weights,
                    color_factor,
                    radius,
                );
            });
    }

    #[cfg(not(feature = "parallel"))]
    {
        for y in 0..h {
            let row = &mut result[y * w..(y + 1) * w];
            bilateral_filter_row(row, y, w, h, src, &spatial_weights, color_factor, radius);
        }
    }

    GrayImage::from_raw(width, height, result).expect("Failed to create filtered image")
}

/// 1行のバイラテラルフィルタ処理
fn bilateral_filter_row(
    row: &mut [u8],
    y: usize,
    w: usize,
    h: usize,
    src: &[u8],
    spatial_weights: &[f32],
    color_factor: f32,
    radius: u32,
) {
    let radius_i = radius as i32;
    let radius_usize = radius as usize;

    for x in 0..w {
        let center_val = src[y * w + x] as f32;
        let mut sum = 0.0f32;
        let mut weight_sum = 0.0f32;

        // カーネル内のピクセルを処理
        for dy in -radius_i..=radius_i {
            let ny = (y as i32 + dy).max(0).min(h as i32 - 1) as usize;

            for dx in -radius_i..=radius_i {
                let nx = (x as i32 + dx).max(0).min(w as i32 - 1) as usize;

                let neighbor_val = src[ny * w + nx] as f32;

                // 空間的重み
                let spatial_idx = ((dy + radius_i) as usize * (2 * radius_usize + 1)
                    + (dx + radius_i) as usize) as usize;
                let spatial_weight = spatial_weights[spatial_idx];

                // 色の重み
                let color_diff = center_val - neighbor_val;
                let color_weight = (color_diff * color_diff * color_factor).exp();

                // 総合重み
                let weight = spatial_weight * color_weight;

                sum += neighbor_val * weight;
                weight_sum += weight;
            }
        }

        // 重み付き平均
        row[x] = if weight_sum > 0.0 {
            (sum / weight_sum).round().clamp(0.0, 255.0) as u8
        } else {
            src[y * w + x]
        };
    }
}

/// 空間的重みを事前計算
fn generate_spatial_weights(radius: u32, sigma: f32) -> Vec<f32> {
    let size = (2 * radius + 1) as usize;
    let mut weights = Vec::with_capacity(size * size);
    let center = radius as f32;

    for y in 0..=2 * radius {
        for x in 0..=2 * radius {
            let dx = (x as f32 - center) / sigma;
            let dy = (y as f32 - center) / sigma;
            let dist_sq = dx * dx + dy * dy;
            let weight = (-0.5 * dist_sq).exp();
            weights.push(weight);
        }
    }

    weights
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bilateral_filter_zero_radius() {
        let alpha = GrayImage::from_fn(10, 10, |_, _| image::Luma([128]));
        let result = bilateral_filter_alpha(&alpha, 5.0, 50.0, 0);
        // 半径0の場合は変化なし
        assert_eq!(result.get_pixel(5, 5).0[0], 128);
    }

    #[test]
    fn test_bilateral_filter_smooth() {
        // ノイズのある画像
        let mut alpha = GrayImage::new(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                let val = if (x + y) % 2 == 0 { 120 } else { 130 };
                alpha.put_pixel(x, y, image::Luma([val]));
            }
        }

        let result = bilateral_filter_alpha(&alpha, 2.0, 10.0, 2);
        // 平滑化される（値が中間に近づく）
        let center = result.get_pixel(5, 5).0[0];
        assert!(center >= 120 && center <= 130);
    }
}

