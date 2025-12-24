//! マットエッジの最適化
//!
//! アルファチャンネルのエッジを最適化して、より自然なエッジ表現を実現

use image::GrayImage;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// エッジ検出方法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeDetectionMethod {
    Sobel,
    Canny,
}

/// マットエッジを最適化
///
/// # Arguments
/// * `alpha` - アルファチャンネル画像
/// * `threshold` - エッジ検出の閾値（0.0 - 1.0）
/// * `smoothness` - エッジの滑らかさ（0.0 - 1.0）
/// * `method` - エッジ検出方法（Sobel/Canny）
///
/// # Returns
/// 最適化されたアルファチャンネル
pub fn optimize_edge(
    alpha: &GrayImage,
    threshold: f32,
    smoothness: f32,
    method: EdgeDetectionMethod,
) -> GrayImage {
    if smoothness <= 0.0 {
        return alpha.clone();
    }

    let (width, height) = alpha.dimensions();
    let w = width as usize;
    let h = height as usize;
    let src = alpha.as_raw();

    // エッジ検出
    let edges = match method {
        EdgeDetectionMethod::Sobel => detect_edges_sobel(alpha, threshold),
        EdgeDetectionMethod::Canny => detect_edges_canny(alpha, threshold * 0.5, threshold, 1.0),
    };

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
pub fn detect_edges_sobel(alpha: &GrayImage, threshold: f32) -> GrayImage {
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
        let result = optimize_edge(&alpha, 0.1, 0.0, EdgeDetectionMethod::Sobel);
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

        let edges = detect_edges_sobel(&alpha, 0.1);
        // エッジが検出される（境界付近の値が0より大きい）
        let edge_val = edges.get_pixel(5, 5).0[0];
        assert!(edge_val > 0);
    }
}

/// Cannyエッジ検出
///
/// # Arguments
/// * `image` - 入力画像
/// * `low_threshold` - 低閾値（0.0 - 1.0）
/// * `high_threshold` - 高閾値（0.0 - 1.0）
/// * `gaussian_sigma` - ガウシアンフィルタのシグマ
///
/// # Returns
/// エッジ検出結果（グレースケール画像）
pub fn detect_edges_canny(
    image: &GrayImage,
    low_threshold: f32,
    high_threshold: f32,
    gaussian_sigma: f32,
) -> GrayImage {
    let (width, height) = image.dimensions();
    let w = width as usize;
    let h = height as usize;

    // 1. ガウシアンフィルタでノイズ除去
    let smoothed = apply_gaussian_blur(image, gaussian_sigma);

    // 2. Sobel演算子で勾配を計算
    let (gradient_magnitude, gradient_direction) = compute_gradient(&smoothed);

    // 3. 非極大値抑制
    let suppressed = non_maximum_suppression(&gradient_magnitude, &gradient_direction, w, h);

    // 4. 二重閾値処理（Hysteresis Thresholding）
    let edges = hysteresis_threshold(&suppressed, low_threshold, high_threshold, w, h);

    GrayImage::from_raw(width, height, edges).expect("Failed to create Canny edge image")
}

/// ガウシアンフィルタを適用
fn apply_gaussian_blur(image: &GrayImage, sigma: f32) -> GrayImage {
    let (width, height) = image.dimensions();
    let w = width as usize;
    let h = height as usize;
    let src = image.as_raw();

    // ガウシアンカーネルサイズ（3シグマルール）
    let kernel_size = ((sigma * 6.0).ceil() as usize) | 1; // 奇数にする
    let kernel_radius = kernel_size / 2;

    let mut result = vec![0u8; (width * height) as usize];

    // ガウシアンカーネルを生成
    let mut kernel = vec![0.0f32; kernel_size];
    let mut sum = 0.0;
    for i in 0..kernel_size {
        let x = i as f32 - kernel_radius as f32;
        let value = (-(x * x) / (2.0 * sigma * sigma)).exp();
        kernel[i] = value;
        sum += value;
    }
    for i in 0..kernel_size {
        kernel[i] /= sum;
    }

    // 水平方向のガウシアンブラー
    let mut temp = vec![0u8; (width * height) as usize];
    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0f32;
            for k in 0..kernel_size {
                let kx = (x as i32 + k as i32 - kernel_radius as i32)
                    .max(0)
                    .min(w as i32 - 1) as usize;
                sum += src[y * w + kx] as f32 * kernel[k];
            }
            temp[y * w + x] = sum.clamp(0.0, 255.0) as u8;
        }
    }

    // 垂直方向のガウシアンブラー
    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0f32;
            for k in 0..kernel_size {
                let ky = (y as i32 + k as i32 - kernel_radius as i32)
                    .max(0)
                    .min(h as i32 - 1) as usize;
                sum += temp[ky * w + x] as f32 * kernel[k];
            }
            result[y * w + x] = sum.clamp(0.0, 255.0) as u8;
        }
    }

    GrayImage::from_raw(width, height, result).expect("Failed to create blurred image")
}

/// 勾配の大きさと方向を計算
fn compute_gradient(image: &GrayImage) -> (Vec<f32>, Vec<f32>) {
    let (width, height) = image.dimensions();
    let w = width as usize;
    let h = height as usize;
    let src = image.as_raw();

    let mut magnitude = vec![0.0f32; (width * height) as usize];
    let mut direction = vec![0.0f32; (width * height) as usize];

    // Sobel演算子
    for y in 1..(h - 1) {
        for x in 1..(w - 1) {
            // Gx
            let gx = -(src[(y - 1) * w + x - 1] as f32)
                + (src[(y - 1) * w + x + 1] as f32)
                - 2.0 * (src[y * w + x - 1] as f32)
                + 2.0 * (src[y * w + x + 1] as f32)
                - (src[(y + 1) * w + x - 1] as f32)
                + (src[(y + 1) * w + x + 1] as f32);

            // Gy
            let gy = -(src[(y - 1) * w + x - 1] as f32)
                - 2.0 * (src[(y - 1) * w + x] as f32)
                - (src[(y - 1) * w + x + 1] as f32)
                + (src[(y + 1) * w + x - 1] as f32)
                + 2.0 * (src[(y + 1) * w + x] as f32)
                + (src[(y + 1) * w + x + 1] as f32);

            let mag = (gx * gx + gy * gy).sqrt();
            let dir = gy.atan2(gx);

            magnitude[y * w + x] = mag;
            direction[y * w + x] = dir;
        }
    }

    (magnitude, direction)
}

/// 非極大値抑制
fn non_maximum_suppression(
    magnitude: &[f32],
    direction: &[f32],
    w: usize,
    h: usize,
) -> Vec<u8> {
    let mut result = vec![0u8; magnitude.len()];

    for y in 1..(h - 1) {
        for x in 1..(w - 1) {
            let idx = y * w + x;
            let mag = magnitude[idx];
            let dir = direction[idx];

            // 方向を0, 45, 90, 135度の4方向に量子化
            let angle = ((dir.to_degrees() + 180.0) % 180.0) / 45.0;
            let sector = (angle as usize) % 4;

            let (dx1, dy1, dx2, dy2) = match sector {
                0 => (1, 0, -1, 0),   // 水平
                1 => (1, -1, -1, 1),  // 45度
                2 => (0, -1, 0, 1),   // 垂直
                3 => (-1, -1, 1, 1),  // 135度
                _ => (1, 0, -1, 0),
            };

            let nx1 = ((x as i32 + dx1).max(0).min(w as i32 - 1)) as usize;
            let ny1 = ((y as i32 + dy1).max(0).min(h as i32 - 1)) as usize;
            let nx2 = ((x as i32 + dx2).max(0).min(w as i32 - 1)) as usize;
            let ny2 = ((y as i32 + dy2).max(0).min(h as i32 - 1)) as usize;

            let mag1 = magnitude[ny1 * w + nx1];
            let mag2 = magnitude[ny2 * w + nx2];

            // 極大値のみ保持
            if mag >= mag1 && mag >= mag2 {
                result[idx] = (mag.min(255.0)) as u8;
            }
        }
    }

    result
}

/// 二重閾値処理（Hysteresis Thresholding）
fn hysteresis_threshold(
    suppressed: &[u8],
    low_threshold: f32,
    high_threshold: f32,
    w: usize,
    h: usize,
) -> Vec<u8> {
    let low = (low_threshold * 255.0) as u8;
    let high = (high_threshold * 255.0) as u8;

    let mut result = vec![0u8; suppressed.len()];
    let mut visited = vec![false; suppressed.len()];

    // 高閾値を超えるピクセルから開始
    for y in 1..(h - 1) {
        for x in 1..(w - 1) {
            let idx = y * w + x;
            if suppressed[idx] >= high && !visited[idx] {
                // エッジ追跡
                trace_edge(suppressed, &mut result, &mut visited, x, y, w, h, low, high);
            }
        }
    }

    result
}

/// エッジ追跡（再帰的）
fn trace_edge(
    suppressed: &[u8],
    result: &mut [u8],
    visited: &mut [bool],
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    low: u8,
    high: u8,
) {
    let idx = y * w + x;
    if visited[idx] || x == 0 || y == 0 || x >= w - 1 || y >= h - 1 {
        return;
    }

    visited[idx] = true;
    result[idx] = 255;

    // 8近傍をチェック
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = (x as i32 + dx).max(0).min(w as i32 - 1) as usize;
            let ny = (y as i32 + dy).max(0).min(h as i32 - 1) as usize;
            let nidx = ny * w + nx;

            if !visited[nidx] && suppressed[nidx] >= low {
                trace_edge(suppressed, result, visited, nx, ny, w, h, low, high);
            }
        }
    }
}

