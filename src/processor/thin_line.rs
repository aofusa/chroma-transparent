//! 細線検出アルゴリズム（髪の毛・毛皮の詳細保持）
//!
//! 細い線（1-2ピクセル幅）を検出してマスクに追加

use image::{GrayImage, RgbaImage};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use crate::color::{rgb_to_hsv, Hsv, Rgb};

/// 細線を検出してマスクに追加
///
/// # Arguments
/// * `image` - 入力画像
/// * `mask` - 既存のマスク
/// * `target_color` - クロマキー対象色
/// * `sensitivity` - 感度（0.0 - 1.0）
/// * `threshold` - 検出閾値（0.0 - 1.0）
///
/// # Returns
/// 細線が追加されたマスク
pub fn detect_thin_lines(
    image: &RgbaImage,
    mask: &GrayImage,
    target_color: &Rgb,
    sensitivity: f32,
    threshold: f32,
) -> GrayImage {
    if sensitivity <= 0.0 {
        return mask.clone();
    }

    let (width, _height) = image.dimensions();
    let w = width as usize;

    // 細線を検出
    let thin_edges = detect_thin_edges(image, target_color, threshold);

    // マスクに統合
    let mut result = mask.clone();
    let result_raw = result.as_mut();
    let thin_edges_raw = thin_edges.as_raw();
    let mask_raw = mask.as_raw();

    #[cfg(feature = "parallel")]
    {
        result_raw
            .par_chunks_mut(w)
            .enumerate()
            .for_each(|(y, row)| {
                for x in 0..w {
                    let idx = y * w + x;
                    let thin_edge_val = thin_edges_raw[idx];
                    let mask_val = mask_raw[idx];

                    // 細線が検出され、マスクが低い（エッジ周辺）場合に追加
                    if thin_edge_val > 0 && mask_val < 200 {
                        // 感度に応じてマスク値を増加
                        let addition = (thin_edge_val as f32 * sensitivity) as u8;
                        row[x] = (mask_val as u32 + addition as u32).min(255) as u8;
                    } else {
                        row[x] = mask_val;
                    }
                }
            });
    }

    #[cfg(not(feature = "parallel"))]
    {
        let (_, height) = image.dimensions();
        let h = height as usize;
        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                let thin_edge_val = thin_edges_raw[idx];
                let mask_val = mask_raw[idx];

                if thin_edge_val > 0 && mask_val < 200 {
                    let addition = (thin_edge_val as f32 * sensitivity) as u8;
                    result_raw[idx] = (mask_val as u32 + addition as u32).min(255) as u8;
                } else {
                    result_raw[idx] = mask_val;
                }
            }
        }
    }

    result
}

/// 細線を検出
fn detect_thin_edges(image: &RgbaImage, target_color: &Rgb, threshold: f32) -> GrayImage {
    let (width, height) = image.dimensions();
    let w = width as usize;
    let h = height as usize;
    let src = image.as_raw();

    let target_hsv = rgb_to_hsv(target_color);
    let threshold_val = (threshold * 255.0) as u8;

    let mut result = vec![0u8; (width * height) as usize];

    #[cfg(feature = "parallel")]
    {
        result
            .par_chunks_mut(w)
            .enumerate()
            .for_each(|(y, row)| {
                if y == 0 || y == h - 1 {
                    return;
                }
                for x in 1..(w - 1) {
                    let _idx = y * w + x;
                    let edge_val = detect_thin_edge_pixel(
                        src,
                        x,
                        y,
                        w,
                        h,
                        target_color,
                        &target_hsv,
                        threshold_val,
                    );
                    row[x] = edge_val;
                }
            });
    }

    #[cfg(not(feature = "parallel"))]
    {
        for y in 1..(h - 1) {
            for x in 1..(w - 1) {
                let idx = y * w + x;
                result[idx] = detect_thin_edge_pixel(
                    src,
                    x,
                    y,
                    w,
                    h,
                    target_color,
                    &target_hsv,
                    threshold_val,
                );
            }
        }
    }

    GrayImage::from_raw(width, height, result).expect("Failed to create thin edge image")
}

/// 1ピクセルの細線検出
fn detect_thin_edge_pixel(
    src: &[u8],
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    _target_color: &Rgb,
    target_hsv: &Hsv,
    threshold: u8,
) -> u8 {
    let center_idx = (y * w + x) * 4;
    let center_rgb = Rgb::new(
        src[center_idx],
        src[center_idx + 1],
        src[center_idx + 2],
    );
    let center_hsv = rgb_to_hsv(&center_rgb);

    // 色類似度をチェック
    let color_similarity = calculate_color_similarity(&center_hsv, target_hsv);
    if color_similarity < threshold as f32 / 255.0 {
        return 0;
    }

    // 細線検出（周囲のピクセルとの比較）
    let mut edge_strength = 0.0f32;
    let mut neighbor_count = 0;

    // 8近傍をチェック
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = (x as i32 + dx).max(0).min(w as i32 - 1) as usize;
            let ny = (y as i32 + dy).max(0).min(h as i32 - 1) as usize;
            let nidx = (ny * w + nx) * 4;

            let neighbor_rgb = Rgb::new(
                src[nidx],
                src[nidx + 1],
                src[nidx + 2],
            );
            let neighbor_hsv = rgb_to_hsv(&neighbor_rgb);

            // 色の差を計算
            let color_diff = calculate_color_similarity(&neighbor_hsv, target_hsv);
            let diff = (color_similarity - color_diff).abs();

            // 中心がクロマ色に近く、周囲が遠い場合、細線として検出
            if color_similarity > 0.7 && color_diff < 0.5 {
                edge_strength += diff;
                neighbor_count += 1;
            }
        }
    }

    if neighbor_count > 0 {
        let avg_edge_strength = edge_strength / neighbor_count as f32;
        (avg_edge_strength * 255.0).clamp(0.0, 255.0) as u8
    } else {
        0
    }
}

/// 色類似度を計算（HSV空間）
fn calculate_color_similarity(hsv1: &Hsv, hsv2: &Hsv) -> f32 {
    // 色相の差（円環距離）
    let h_diff = {
        let diff = (hsv1.h - hsv2.h).abs();
        if diff > 180.0 {
            360.0 - diff
        } else {
            diff
        }
    };

    // 彩度と明度の差
    let s_diff = (hsv1.s - hsv2.s).abs();
    let v_diff = (hsv1.v - hsv2.v).abs();

    // 重み付き距離（色相0.5、彩度0.3、明度0.2）
    let distance = (h_diff * 0.5 + s_diff * 300.0 * 0.3 + v_diff * 100.0 * 0.2) / 100.0;

    // 類似度に変換（0-1、1が最も類似）
    1.0 - (distance / 2.0).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_detect_thin_lines_zero_sensitivity() {
        let image = RgbaImage::from_fn(10, 10, |_, _| Rgba([0, 255, 0, 255]));
        let mask = GrayImage::from_fn(10, 10, |_, _| image::Luma([128]));
        let target = Rgb::new(0, 255, 0);

        let result = detect_thin_lines(&image, &mask, &target, 0.0, 0.3);
        // 感度0の場合は変化なし
        assert_eq!(result.get_pixel(5, 5).0[0], 128);
    }

    #[test]
    fn test_detect_thin_edges() {
        let mut image = RgbaImage::new(10, 10);
        // 細い線を模擬
        for y in 0..10 {
            for x in 0..10 {
                if x == 5 {
                    image.put_pixel(x, y, Rgba([0, 255, 0, 255])); // 細い緑の線
                } else {
                    image.put_pixel(x, y, Rgba([255, 0, 0, 255])); // 赤の背景
                }
            }
        }

        let target = Rgb::new(0, 255, 0);
        let edges = detect_thin_edges(&image, &target, 0.3);

        // 細線が検出される
        let edge_val = edges.get_pixel(5, 5).0[0];
        assert!(edge_val > 0);
    }
}

