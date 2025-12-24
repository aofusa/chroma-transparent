//! 自動パラメータ推定
//!
//! 画像を分析して最適なパラメータを自動設定

use image::RgbaImage;

use crate::color::Rgb;
use crate::config::ProcessConfig;

/// 推定されたパラメータ
#[derive(Debug, Clone)]
pub struct EstimatedParameters {
    pub tolerance: f32,
    pub feather_amount: u32,
    pub despill_strength: f32,
    pub erode_iterations: u32,
    pub dilate_iterations: u32,
    pub adaptive_tolerance_enabled: bool,
    pub multiscale_enabled: bool,
    pub edge_optimization_enabled: bool,
}

impl Default for EstimatedParameters {
    fn default() -> Self {
        Self {
            tolerance: 0.3,
            feather_amount: 5,
            despill_strength: 0.7,
            erode_iterations: 0,
            dilate_iterations: 1,
            adaptive_tolerance_enabled: false,
            multiscale_enabled: false,
            edge_optimization_enabled: false,
        }
    }
}

/// 画像から最適なパラメータを推定
///
/// # Arguments
/// * `image` - 入力画像
/// * `chroma_color` - クロマキー対象色
///
/// # Returns
/// 推定されたパラメータ
pub fn estimate_parameters(image: &RgbaImage, chroma_color: &Rgb) -> EstimatedParameters {
    let (width, height) = image.dimensions();
    let _total_pixels = (width * height) as usize;

    // 1. 色分布分析
    let color_stats = analyze_color_distribution(image, chroma_color);

    // 2. コントラスト分析
    let contrast = calculate_contrast(image);

    // 3. エッジ密度分析
    let edge_density = calculate_edge_density(image);

    // 4. 照明分析
    let lighting_variance = calculate_lighting_variance(image);

    // 5. ヒューリスティックルール適用
    let mut params = EstimatedParameters::default();

    // Tolerance推定
    // 色分布が広い → toleranceを拡大
    if color_stats.std_dev > 50.0 {
        params.tolerance = (0.3 + (color_stats.std_dev - 50.0) / 200.0).min(0.8);
    } else {
        params.tolerance = (0.2 + color_stats.std_dev / 250.0).max(0.1);
    }

    // Feather推定
    // コントラストが高い → feather_amountを増やす
    if contrast > 0.5 {
        params.feather_amount = ((contrast * 10.0) as u32).min(20);
    } else {
        params.feather_amount = ((contrast * 5.0) as u32).max(2);
    }

    // Despill強度推定
    // エッジ密度が高い → despill_strengthを増やす
    if edge_density > 0.3 {
        params.despill_strength = (0.5 + edge_density * 0.5).min(1.0);
    } else {
        params.despill_strength = (0.3 + edge_density * 0.4).max(0.2);
    }

    // モルフォロジー演算推定
    // エッジ密度が高い → erode_iterationsを増やす
    if edge_density > 0.4 {
        params.erode_iterations = ((edge_density * 3.0) as u32).min(3);
        params.dilate_iterations = ((edge_density * 2.0) as u32).min(2);
    } else {
        params.erode_iterations = 0;
        params.dilate_iterations = 1;
    }

    // 適応的許容範囲の有効化
    // 照明ムラがある → adaptive_toleranceを有効化
    if lighting_variance > 0.15 {
        params.adaptive_tolerance_enabled = true;
    }

    // マルチスケール処理の有効化
    // エッジ密度が高い → multiscaleを有効化
    if edge_density > 0.35 {
        params.multiscale_enabled = true;
    }

    // エッジ最適化の有効化
    // コントラストが高い → edge_optimizationを有効化
    if contrast > 0.4 {
        params.edge_optimization_enabled = true;
    }

    params
}

/// 色分布統計
struct ColorStats {
    std_dev: f32,
}

/// 色分布を分析
fn analyze_color_distribution(image: &RgbaImage, target_color: &Rgb) -> ColorStats {
    let (width, height) = image.dimensions();
    let mut distances = Vec::new();

    // サンプリング（パフォーマンスのため全ピクセルではなくサンプリング）
    let sample_rate = (width * height / 10000).max(1);
    for y in (0..height).step_by(sample_rate as usize) {
        for x in (0..width).step_by(sample_rate as usize) {
            let pixel = image.get_pixel(x, y);
            let pixel_rgb = Rgb::new(pixel[0], pixel[1], pixel[2]);

            let dr = (pixel_rgb.r as f32 - target_color.r as f32) as f32;
            let dg = (pixel_rgb.g as f32 - target_color.g as f32) as f32;
            let db = (pixel_rgb.b as f32 - target_color.b as f32) as f32;
            let distance = (dr * dr + dg * dg + db * db).sqrt();

            distances.push(distance);
        }
    }

    if distances.is_empty() {
        return ColorStats {
            std_dev: 0.0,
        };
    }

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

    ColorStats { std_dev }
}

/// コントラストを計算
fn calculate_contrast(image: &RgbaImage) -> f32 {
    let (width, height) = image.dimensions();
    let mut luminances = Vec::new();

    // サンプリング
    let sample_rate = (width * height / 5000).max(1);
    for y in (0..height).step_by(sample_rate as usize) {
        for x in (0..width).step_by(sample_rate as usize) {
            let pixel = image.get_pixel(x, y);
            // 明度を計算（YUVのY成分）
            let luminance = 0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32;
            luminances.push(luminance);
        }
    }

    if luminances.is_empty() {
        return 0.0;
    }

    let mean = luminances.iter().sum::<f32>() / luminances.len() as f32;
    let variance = luminances
        .iter()
        .map(|l| {
            let diff = l - mean;
            diff * diff
        })
        .sum::<f32>()
        / luminances.len() as f32;
    let std_dev = variance.sqrt();

    // コントラストを0-1の範囲に正規化
    (std_dev / 128.0).min(1.0)
}

/// エッジ密度を計算
fn calculate_edge_density(image: &RgbaImage) -> f32 {
    let (width, height) = image.dimensions();
    let w = width as usize;
    let h = height as usize;
    let src = image.as_raw();

    let mut edge_count = 0;
    let mut total_pixels = 0;

    // 簡易的なエッジ検出（Sobel演算子の簡易版）
    for y in 1..(h - 1) {
        for x in 1..(w - 1) {
            total_pixels += 1;

            let idx = (y * w + x) * 4;
            let center_lum = 0.299 * src[idx] as f32
                + 0.587 * src[idx + 1] as f32
                + 0.114 * src[idx + 2] as f32;

            // 右隣
            let right_idx = (y * w + x + 1) * 4;
            let right_lum = 0.299 * src[right_idx] as f32
                + 0.587 * src[right_idx + 1] as f32
                + 0.114 * src[right_idx + 2] as f32;

            // 下隣
            let bottom_idx = ((y + 1) * w + x) * 4;
            let bottom_lum = 0.299 * src[bottom_idx] as f32
                + 0.587 * src[bottom_idx + 1] as f32
                + 0.114 * src[bottom_idx + 2] as f32;

            let diff_x = (center_lum - right_lum).abs();
            let diff_y = (center_lum - bottom_lum).abs();
            let edge_strength = (diff_x + diff_y) / 2.0;

            if edge_strength > 20.0 {
                edge_count += 1;
            }
        }
    }

    if total_pixels == 0 {
        return 0.0;
    }

    (edge_count as f32 / total_pixels as f32).min(1.0)
}

/// 照明の分散を計算
fn calculate_lighting_variance(image: &RgbaImage) -> f32 {
    let (width, height) = image.dimensions();
    let mut luminances = Vec::new();

    // サンプリング
    let sample_rate = (width * height / 5000).max(1);
    for y in (0..height).step_by(sample_rate as usize) {
        for x in (0..width).step_by(sample_rate as usize) {
            let pixel = image.get_pixel(x, y);
            let luminance = 0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32;
            luminances.push(luminance);
        }
    }

    if luminances.is_empty() {
        return 0.0;
    }

    let mean = luminances.iter().sum::<f32>() / luminances.len() as f32;
    let variance = luminances
        .iter()
        .map(|l| {
            let diff = l - mean;
            diff * diff
        })
        .sum::<f32>()
        / luminances.len() as f32;
    let std_dev = variance.sqrt();

    // 分散を0-1の範囲に正規化
    (std_dev / 128.0).min(1.0)
}

/// 推定パラメータをProcessConfigに適用
pub fn apply_estimated_parameters(config: &mut ProcessConfig, estimated: &EstimatedParameters) {
    config.tolerance = estimated.tolerance;
    config.feather_amount = estimated.feather_amount;
    config.despill_strength = estimated.despill_strength;
    config.erode_iterations = estimated.erode_iterations;
    config.dilate_iterations = estimated.dilate_iterations;
    // 注意: adaptive_tolerance_enabled等のフィールドはProcessConfigに追加する必要がある
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_estimate_parameters() {
        let mut image = RgbaImage::new(100, 100);
        // 高コントラストの画像
        for y in 0..100 {
            for x in 0..100 {
                if (x + y) % 20 < 10 {
                    image.put_pixel(x, y, Rgba([0, 255, 0, 255])); // 緑
                } else {
                    image.put_pixel(x, y, Rgba([255, 0, 0, 255])); // 赤
                }
            }
        }

        let target = Rgb::new(0, 255, 0);
        let params = estimate_parameters(&image, &target);

        // パラメータが適切に推定されていることを確認
        assert!(params.tolerance >= 0.0 && params.tolerance <= 1.0);
        assert!(params.feather_amount <= 50);
        assert!(params.despill_strength >= 0.0 && params.despill_strength <= 1.0);
    }
}

