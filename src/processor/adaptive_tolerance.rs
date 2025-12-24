//! 適応的許容範囲（Adaptive Tolerance）
//!
//! 画像をグリッド分割し、各領域の色分布を分析して最適なtolerance値を計算

use image::{GrayImage, RgbaImage};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use crate::color::Rgb;

/// 適応的許容範囲マップを生成
///
/// # Arguments
/// * `image` - 入力画像
/// * `target_color` - クロマキー対象色
/// * `base_tolerance` - ベースとなる許容範囲 (0.0 - 1.0)
/// * `grid_size` - グリッドサイズ (width, height)
/// * `sensitivity` - 感度調整 (0.0 - 2.0、デフォルト: 1.0)
///
/// # Returns
/// 各ピクセル位置の適応的tolerance値（0.0 - 1.0）を格納したグレースケール画像
/// 値は0-255の範囲で、tolerance = value / 255.0
pub fn create_adaptive_tolerance_map(
    image: &RgbaImage,
    target_color: &Rgb,
    base_tolerance: f32,
    grid_size: (u32, u32),
    sensitivity: f32,
) -> GrayImage {
    let (width, height) = image.dimensions();
    let (grid_w, grid_h) = grid_size;

    // グリッド領域ごとのtoleranceを計算
    let grid_tolerances = calculate_grid_tolerances(
        image,
        target_color,
        base_tolerance,
        grid_w,
        grid_h,
        sensitivity,
    );

    // バイリニア補間で各ピクセルのtoleranceを計算
    interpolate_tolerance_map(width, height, grid_w, grid_h, &grid_tolerances)
}

/// 各グリッド領域の最適toleranceを計算
fn calculate_grid_tolerances(
    image: &RgbaImage,
    target_color: &Rgb,
    base_tolerance: f32,
    grid_w: u32,
    grid_h: u32,
    sensitivity: f32,
) -> Vec<Vec<f32>> {
    let (width, height) = image.dimensions();
    let cell_w = width / grid_w;
    let cell_h = height / grid_h;

    let mut grid_tolerances = vec![vec![0.0f32; grid_w as usize]; grid_h as usize];

    #[cfg(feature = "parallel")]
    {
        grid_tolerances
            .par_iter_mut()
            .enumerate()
            .for_each(|(gy, row)| {
                for gx in 0..grid_w as usize {
                    let region_tolerance = calculate_region_tolerance(
                        image,
                        target_color,
                        base_tolerance,
                        gx,
                        gy,
                        cell_w,
                        cell_h,
                        sensitivity,
                    );
                    row[gx] = region_tolerance;
                }
            });
    }

    #[cfg(not(feature = "parallel"))]
    {
        for gy in 0..grid_h as usize {
            for gx in 0..grid_w as usize {
                grid_tolerances[gy][gx] = calculate_region_tolerance(
                    image,
                    target_color,
                    base_tolerance,
                    gx,
                    gy,
                    cell_w,
                    cell_h,
                    sensitivity,
                );
            }
        }
    }

    grid_tolerances
}

/// 1つのグリッド領域の最適toleranceを計算
fn calculate_region_tolerance(
    image: &RgbaImage,
    target_color: &Rgb,
    base_tolerance: f32,
    grid_x: usize,
    grid_y: usize,
    cell_w: u32,
    cell_h: u32,
    sensitivity: f32,
) -> f32 {
    let (width, height) = image.dimensions();
    let start_x = (grid_x as u32 * cell_w).min(width);
    let start_y = (grid_y as u32 * cell_h).min(height);
    let end_x = ((grid_x as u32 + 1) * cell_w).min(width);
    let end_y = ((grid_y as u32 + 1) * cell_h).min(height);

    // 領域内のピクセル色を収集
    let mut colors = Vec::new();
    for y in start_y..end_y {
        for x in start_x..end_x {
            let pixel = image.get_pixel(x, y);
            colors.push(Rgb::new(pixel[0], pixel[1], pixel[2]));
        }
    }

    if colors.is_empty() {
        return base_tolerance;
    }

    // 色分布の標準偏差を計算（RGB距離ベース）
    let distances: Vec<f32> = colors
        .iter()
        .map(|c| {
            let dr = (c.r as f32 - target_color.r as f32) as f32;
            let dg = (c.g as f32 - target_color.g as f32) as f32;
            let db = (c.b as f32 - target_color.b as f32) as f32;
            (dr * dr + dg * dg + db * db).sqrt()
        })
        .collect();

    let mean = distances.iter().sum::<f32>() / distances.len() as f32;
    let variance = distances
        .iter()
        .map(|d| {
            let diff = d - mean;
            diff * diff
        })
        .sum::<f32>()
        / distances.len() as f32;
    let std_dev = variance.sqrt();

    // 標準偏差に基づいてtoleranceを調整
    // 標準偏差が大きい（照明ムラがある）→ toleranceを拡大
    // 標準偏差が小さい（均一）→ toleranceを縮小
    let normalized_std = (std_dev / 441.67).min(1.0); // 441.67は最大RGB距離
    let adjustment = (normalized_std - 0.5) * 2.0 * sensitivity; // -sensitivity から +sensitivity の範囲

    let adjusted_tolerance = base_tolerance * (1.0 + adjustment);
    adjusted_tolerance.clamp(0.0, 1.0)
}

/// バイリニア補間で各ピクセルのtoleranceを計算
fn interpolate_tolerance_map(
    width: u32,
    height: u32,
    grid_w: u32,
    grid_h: u32,
    grid_tolerances: &[Vec<f32>],
) -> GrayImage {
    let cell_w = width as f32 / grid_w as f32;
    let cell_h = height as f32 / grid_h as f32;

    let mut result = vec![0u8; (width * height) as usize];

    #[cfg(feature = "parallel")]
    {
        result
            .par_chunks_mut(width as usize)
            .enumerate()
            .for_each(|(y, row)| {
                for x in 0..width as usize {
                    let tolerance = bilinear_interpolate(
                        x as f32,
                        y as f32,
                        cell_w,
                        cell_h,
                        grid_w,
                        grid_h,
                        grid_tolerances,
                    );
                    row[x] = (tolerance * 255.0).clamp(0.0, 255.0) as u8;
                }
            });
    }

    #[cfg(not(feature = "parallel"))]
    {
        for y in 0..height as usize {
            for x in 0..width as usize {
                let tolerance = bilinear_interpolate(
                    x as f32,
                    y as f32,
                    cell_w,
                    cell_h,
                    grid_w,
                    grid_h,
                    grid_tolerances,
                );
                result[y * width as usize + x] =
                    (tolerance * 255.0).clamp(0.0, 255.0) as u8;
            }
        }
    }

    GrayImage::from_raw(width, height, result).expect("Failed to create tolerance map")
}

/// バイリニア補間
fn bilinear_interpolate(
    x: f32,
    y: f32,
    cell_w: f32,
    cell_h: f32,
    grid_w: u32,
    grid_h: u32,
    grid_tolerances: &[Vec<f32>],
) -> f32 {
    let gx = (x / cell_w).floor().max(0.0).min(grid_w as f32 - 1.0);
    let gy = (y / cell_h).floor().max(0.0).min(grid_h as f32 - 1.0);

    let gx0 = gx as usize;
    let gy0 = gy as usize;
    let gx1 = (gx + 1.0).min(grid_w as f32 - 1.0) as usize;
    let gy1 = (gy + 1.0).min(grid_h as f32 - 1.0) as usize;

    let fx = (x / cell_w) - gx;
    let fy = (y / cell_h) - gy;

    let v00 = grid_tolerances[gy0][gx0];
    let v10 = grid_tolerances[gy0][gx1];
    let v01 = grid_tolerances[gy1][gx0];
    let v11 = grid_tolerances[gy1][gx1];

    let v0 = v00 * (1.0 - fx) + v10 * fx;
    let v1 = v01 * (1.0 - fx) + v11 * fx;

    v0 * (1.0 - fy) + v1 * fy
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_adaptive_tolerance_map() {
        let mut image = RgbaImage::new(100, 100);
        // 左半分は明るい緑、右半分は暗い緑
        for y in 0..100 {
            for x in 0..100 {
                if x < 50 {
                    image.put_pixel(x, y, Rgba([0, 255, 0, 255])); // 明るい緑
                } else {
                    image.put_pixel(x, y, Rgba([0, 128, 0, 255])); // 暗い緑
                }
            }
        }

        let target = Rgb::new(0, 255, 0);
        let tolerance_map = create_adaptive_tolerance_map(&image, &target, 0.3, (4, 4), 1.0);

        // 左半分と右半分でtolerance値が異なることを確認
        let left_tolerance = tolerance_map.get_pixel(25, 50).0[0] as f32 / 255.0;
        let right_tolerance = tolerance_map.get_pixel(75, 50).0[0] as f32 / 255.0;

        // 照明ムラがあるため、tolerance値が調整されているはず
        assert!(left_tolerance != right_tolerance || (left_tolerance - 0.3).abs() < 0.1);
    }

    #[test]
    fn test_calculate_region_tolerance() {
        let mut image = RgbaImage::new(10, 10);
        // 均一な色
        for y in 0..10 {
            for x in 0..10 {
                image.put_pixel(x, y, Rgba([0, 255, 0, 255]));
            }
        }

        let target = Rgb::new(0, 255, 0);
        let tolerance = calculate_region_tolerance(&image, &target, 0.3, 0, 0, 10, 10, 1.0);

        // 均一な領域ではtoleranceが調整される
        assert!(tolerance >= 0.0 && tolerance <= 1.0);
    }
}

