//! マットエッジの最適化
//!
//! アルファチャンネルのエッジを最適化して、より自然なエッジ表現を実現

use image::GrayImage;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// マットエッジを最適化
///
/// # Arguments
/// * `alpha` - アルファチャンネル画像
/// * `threshold` - エッジ検出の閾値（0.0 - 1.0）
/// * `smoothness` - エッジの滑らかさ（0.0 - 1.0）
///
/// # Returns
/// 最適化されたアルファチャンネル
pub fn optimize_edge(alpha: &GrayImage, threshold: f32, smoothness: f32) -> GrayImage {
    if smoothness <= 0.0 {
        return alpha.clone();
    }

    let (width, height) = alpha.dimensions();
    let w = width as usize;
    let h = height as usize;
    let src = alpha.as_raw();

    // エッジ検出（Sobel演算子）
    let edges = detect_edges(alpha, threshold);

    // エッジ周辺でのアルファ値の補間最適化
    let mut result = vec![0u8; (width * height) as usize];

    #[cfg(feature = "parallel")]
    {
        result
            .par_chunks_mut(w)
            .enumerate()
            .for_each(|(y, row)| {
                optimize_edge_row(row, y, w, h, src, &edges, smoothness);
            });
    }

    #[cfg(not(feature = "parallel"))]
    {
        for y in 0..h {
            let row = &mut result[y * w..(y + 1) * w];
            optimize_edge_row(row, y, w, h, src, &edges, smoothness);
        }
    }

    GrayImage::from_raw(width, height, result).expect("Failed to create optimized alpha")
}

/// エッジ検出（Sobel演算子）
fn detect_edges(alpha: &GrayImage, threshold: f32) -> GrayImage {
    let (width, height) = alpha.dimensions();
    let w = width as usize;
    let h = height as usize;
    let src = alpha.as_raw();
    let mut result = vec![0u8; (width * height) as usize];

    // Sobel演算子のカーネル
    // Gx = [-1  0  1]    Gy = [-1 -2 -1]
    //      [-2  0  2]         [ 0  0  0]
    //      [-1  0  1]         [ 1  2  1]

    for y in 1..(h - 1) {
        for x in 1..(w - 1) {
            // Gx
            let gx = -(src[(y - 1) * w + x - 1] as i16)
                + (src[(y - 1) * w + x + 1] as i16)
                - 2 * (src[y * w + x - 1] as i16)
                + 2 * (src[y * w + x + 1] as i16)
                - (src[(y + 1) * w + x - 1] as i16)
                + (src[(y + 1) * w + x + 1] as i16);

            // Gy
            let gy = -(src[(y - 1) * w + x - 1] as i16)
                - 2 * (src[(y - 1) * w + x] as i16)
                - (src[(y - 1) * w + x + 1] as i16)
                + (src[(y + 1) * w + x - 1] as i16)
                + 2 * (src[(y + 1) * w + x] as i16)
                + (src[(y + 1) * w + x + 1] as i16);

            // エッジ強度
            let magnitude = ((gx * gx + gy * gy) as f32).sqrt();
            let normalized = (magnitude / 255.0).min(1.0);

            // 閾値チェック
            if normalized > threshold {
                result[y * w + x] = (normalized * 255.0) as u8;
            } else {
                result[y * w + x] = 0;
            }
        }
    }

    GrayImage::from_raw(width, height, result).expect("Failed to create edge image")
}

/// 1行のエッジ最適化処理
fn optimize_edge_row(
    row: &mut [u8],
    y: usize,
    w: usize,
    h: usize,
    src: &[u8],
    edges: &GrayImage,
    smoothness: f32,
) {
    let edge_raw = edges.as_raw();

    for x in 0..w {
        let idx = y * w + x;
        let alpha_val = src[idx] as f32;
        let edge_strength = edge_raw[idx] as f32 / 255.0;

        if edge_strength > 0.0 {
            // エッジ周辺での補間最適化
            // 周囲のピクセルの平均を計算
            let mut sum = 0.0f32;
            let mut count = 0;

            for dy in -1i32..=1 {
                let ny = (y as i32 + dy).max(0).min(h as i32 - 1) as usize;
                for dx in -1i32..=1 {
                    let nx = (x as i32 + dx).max(0).min(w as i32 - 1) as usize;
                    sum += src[ny * w + nx] as f32;
                    count += 1;
                }
            }

            let avg = sum / count as f32;

            // エッジ強度に応じて補間
            let optimized = alpha_val * (1.0 - edge_strength * smoothness)
                + avg * (edge_strength * smoothness);

            row[x] = optimized.clamp(0.0, 255.0) as u8;
        } else {
            row[x] = src[idx];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimize_edge_zero_smoothness() {
        let alpha = GrayImage::from_fn(10, 10, |_, _| image::Luma([128]));
        let result = optimize_edge(&alpha, 0.1, 0.0);
        // 滑らかさ0の場合は変化なし
        assert_eq!(result.get_pixel(5, 5).0[0], 128);
    }

    #[test]
    fn test_detect_edges() {
        // 高コントラストの画像（エッジがある）
        let mut alpha = GrayImage::new(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                if x < 5 {
                    alpha.put_pixel(x, y, image::Luma([0]));
                } else {
                    alpha.put_pixel(x, y, image::Luma([255]));
                }
            }
        }

        let edges = detect_edges(&alpha, 0.1);
        // エッジが検出される（境界付近の値が0より大きい）
        let edge_val = edges.get_pixel(5, 5).0[0];
        assert!(edge_val > 0);
    }
}

