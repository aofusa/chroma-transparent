//! 処理パイプライン（最適化版）
//!
//! パフォーマンス改善:
//! - A: Rayon並列処理（各処理ステップ内）
//! - B: RGB距離による事前フィルタリング（mask生成時）
//! - C: ダブルバッファリング（morphology）
//! - D: バッファ直接操作（各処理ステップ）
//! - E: SIMD最適化（alpha処理）
//! - F: LUT（mask生成時）
//! - G: インプレース処理（despill, alpha適用）

use image::RgbaImage;
use log::debug;

use crate::config::ProcessConfig;
use crate::processor::alpha::apply_alpha_inplace;
use crate::processor::{
    apply_estimated_parameters, bilateral_filter_alpha, create_adaptive_tolerance_map,
    create_alpha_from_mask, create_chroma_mask, create_multi_chroma_mask,
    create_multiscale_mask, despill, detect_thin_lines, dilate, erode, estimate_parameters,
    feather_alpha, optimize_edge, remove_shadows, sharpen,
};

/// クロマキー処理パイプライン
pub struct ChromaPipeline {
    config: ProcessConfig,
}

impl ChromaPipeline {
    /// 新しいパイプラインを作成
    pub fn new(config: ProcessConfig) -> Self {
        Self { config }
    }

    /// 画像を処理（最適化版）
    ///
    /// すべての改善案（A〜G）を適用した高速処理
    pub fn process(&self, image: &RgbaImage) -> RgbaImage {
        debug!("Processing started (optimized)...");

        // 0. 自動パラメータ推定（有効時）
        let mut config = self.config.clone();
        if config.auto_params_enabled {
            let estimated = estimate_parameters(image, &config.chroma_color);
            apply_estimated_parameters(&mut config, &estimated);
            debug!("Auto parameters estimated");
        }

        // 1. クロマキーマスク生成（改善A, B, D, F適用）
        // 適応的許容範囲マップを生成（有効時）
        let tolerance_map = if config.adaptive_tolerance_enabled {
            Some(create_adaptive_tolerance_map(
                image,
                &config.chroma_color,
                config.tolerance,
                (config.adaptive_tolerance_grid_w, config.adaptive_tolerance_grid_h),
                config.adaptive_tolerance_sensitivity,
            ))
        } else {
            None
        };
        
        // マスク生成（適応的許容範囲マップを使用）
        let mask = if config.multiscale_enabled {
            // マルチスケール処理（適応的許容範囲は未対応、将来的に拡張可能）
            create_multiscale_mask(
                image,
                &config.chroma_color,
                config.tolerance,
                config.multiscale_levels,
                config.multiscale_scale_factor,
                config.color_space,
            )
        } else if let Some(ref multi_colors) = config.multi_colors {
            // 多色検出（適応的許容範囲は未対応、将来的に拡張可能）
            create_multi_chroma_mask(image, multi_colors, config.color_space)
        } else {
            // 単色検出（適応的許容範囲対応）
            create_chroma_mask(
                image,
                &config.chroma_color,
                config.tolerance,
                config.color_space,
                tolerance_map.as_ref(),
            )
        };
        debug!("Mask generation completed");

        // 1.5. 細線検出（有効時）
        let mask = if config.thin_line_detection_enabled {
            let result = detect_thin_lines(
                image,
                &mask,
                &config.chroma_color,
                config.thin_line_sensitivity,
                config.thin_line_threshold,
            );
            debug!("Thin line detection completed");
            result
        } else {
            mask
        };

        // 2. 影の処理（有効時）
        let mask = if config.shadow_removal_enabled {
            let result = remove_shadows(
                &mask,
                image,
                config.shadow_threshold,
                config.shadow_removal_strength,
            );
            debug!(
                "Shadow removal completed (threshold={}, strength={})",
                config.shadow_threshold, config.shadow_removal_strength
            );
            result
        } else {
            mask
        };

        // 3. モルフォロジー演算（改善A, C, D適用）
        let mask = if config.erode_iterations > 0 {
            let result = erode(&mask, config.erode_iterations);
            debug!(
                "Erode completed ({} iterations)",
                config.erode_iterations
            );
            result
        } else {
            mask
        };

        let mask = if config.dilate_iterations > 0 {
            let result = dilate(&mask, config.dilate_iterations);
            debug!(
                "Dilate completed ({} iterations)",
                config.dilate_iterations
            );
            result
        } else {
            mask
        };

        // 4. アルファチャンネル生成（改善E適用：SIMD反転）
        let alpha_channel = create_alpha_from_mask(&mask);
        debug!("Alpha channel generation completed");

        // 4.5. マットエッジ最適化（有効時）
        let alpha_channel = if config.edge_optimization_enabled {
            let result = optimize_edge(
                &alpha_channel,
                config.edge_threshold,
                config.edge_smoothness,
                config.edge_detection_method,
            );
            debug!(
                "Edge optimization completed (threshold={}, smoothness={}, method={:?})",
                config.edge_threshold, config.edge_smoothness, config.edge_detection_method
            );
            result
        } else {
            alpha_channel
        };

        // 5. フェザリング（改善A, D適用）
        let alpha_channel = if config.feather_amount > 0 {
            let result = feather_alpha(&alpha_channel, config.feather_amount);
            debug!(
                "Feathering completed (amount={})",
                config.feather_amount
            );
            result
        } else {
            alpha_channel
        };

        // 5.5. バイラテラルフィルタ（有効時）
        let alpha_channel = if config.bilateral_enabled {
            let result = bilateral_filter_alpha(
                &alpha_channel,
                config.bilateral_spatial_sigma,
                config.bilateral_color_sigma,
                config.bilateral_radius,
            );
            debug!(
                "Bilateral filter completed (spatial_sigma={}, color_sigma={})",
                config.bilateral_spatial_sigma, config.bilateral_color_sigma
            );
            result
        } else {
            alpha_channel
        };

        // 6. デスピル処理（改善A, D, G適用：インプレース）
        let mut result = image.clone();
        if config.despill_strength > 0.0 {
            despill(
                &mut result,
                &mask,
                config.despill_strength,
                &config.chroma_color,
                config.despill_method,
            );
            debug!(
                "Despill completed (strength={}, method={:?})",
                config.despill_strength, config.despill_method
            );
        }

        // 7. エッジシャープニング（有効時）
        if config.sharpen_enabled {
            result = sharpen(
                &result,
                config.sharpen_amount,
                config.sharpen_radius,
                config.sharpen_threshold,
            );
            debug!(
                "Sharpen completed (amount={}, radius={})",
                config.sharpen_amount, config.sharpen_radius
            );
        }

        // 8. アルファチャンネル適用（改善E, G適用：SIMD + インプレース）
        apply_alpha_inplace(&mut result, &alpha_channel);
        debug!("Alpha channel applied");

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Rgb;
    use image::Rgba;

    fn create_test_image() -> RgbaImage {
        let mut image = RgbaImage::new(10, 10);

        // 緑の背景
        for y in 0..10 {
            for x in 0..10 {
                image.put_pixel(x, y, Rgba([0, 255, 0, 255]));
            }
        }

        // 中央に赤い四角
        for y in 3..7 {
            for x in 3..7 {
                image.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }

        image
    }

    #[test]
    fn test_pipeline_basic() {
        let config = ProcessConfig {
            chroma_color: Rgb::new(0, 255, 0),
            tolerance: 0.3,
            feather_amount: 0,
            despill_strength: 0.0,
            erode_iterations: 0,
            dilate_iterations: 0,
            verbose: false,
            ..Default::default()
        };

        let pipeline = ChromaPipeline::new(config);
        let image = create_test_image();
        let result = pipeline.process(&image);

        // 緑の部分は透明になる
        assert_eq!(result.get_pixel(0, 0).0[3], 0);

        // 赤い部分は不透明のまま
        assert_eq!(result.get_pixel(5, 5).0[3], 255);
    }

    #[test]
    fn test_pipeline_with_dilate() {
        let config = ProcessConfig {
            chroma_color: Rgb::new(0, 255, 0),
            tolerance: 0.3,
            feather_amount: 0,
            despill_strength: 0.0,
            erode_iterations: 0,
            dilate_iterations: 1,
            verbose: false,
            ..Default::default()
        };

        let pipeline = ChromaPipeline::new(config);
        let image = create_test_image();
        let result = pipeline.process(&image);

        // 処理が正常に完了すること
        assert!(result.width() == 10);
        assert!(result.height() == 10);
    }
}

