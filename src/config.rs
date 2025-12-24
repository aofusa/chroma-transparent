//! 処理パラメータ設定

#[cfg(feature = "cli")]
use crate::cli::Args;
use crate::color::{ColorSpace, Rgb};
use crate::error::{ChromaError, Result};
use crate::processor::{DespillMethod, EdgeDetectionMethod};

/// 多色検出用の色設定
#[derive(Debug, Clone)]
pub struct ColorConfig {
    pub color: Rgb,
    pub tolerance: f32,
}

/// クロマキー処理の設定パラメータ
#[derive(Debug, Clone)]
pub struct ProcessConfig {
    /// クロマキー対象色
    pub chroma_color: Rgb,
    /// 色の許容範囲 (0.0 - 1.0)
    pub tolerance: f32,
    /// フェザリング量 (0 - 50)
    pub feather_amount: u32,
    /// デスピル強度 (0.0 - 1.0)
    pub despill_strength: f32,
    /// 収縮回数 (0 - 10)
    pub erode_iterations: u32,
    /// 膨張回数 (0 - 10)
    pub dilate_iterations: u32,
    /// 詳細ログ出力
    pub verbose: bool,
    
    // 新規フィールド
    /// 多色検出設定（Noneの場合は単色検出）
    pub multi_colors: Option<Vec<ColorConfig>>,
    /// 色空間
    pub color_space: ColorSpace,
    
    /// バイラテラルフィルタ有効/無効
    pub bilateral_enabled: bool,
    /// バイラテラルフィルタ: 空間的重みの標準偏差
    pub bilateral_spatial_sigma: f32,
    /// バイラテラルフィルタ: 色の重みの標準偏差
    pub bilateral_color_sigma: f32,
    /// バイラテラルフィルタ: カーネル半径
    pub bilateral_radius: u32,
    
    /// マルチスケール処理有効/無効
    pub multiscale_enabled: bool,
    /// マルチスケール処理: スケールレベル数
    pub multiscale_levels: u32,
    /// マルチスケール処理: スケール係数
    pub multiscale_scale_factor: f32,
    
    /// マットエッジ最適化有効/無効
    pub edge_optimization_enabled: bool,
    /// マットエッジ最適化: エッジ検出の閾値
    pub edge_threshold: f32,
    /// マットエッジ最適化: エッジの滑らかさ
    pub edge_smoothness: f32,
    
    /// 影の処理有効/無効
    pub shadow_removal_enabled: bool,
    /// 影の処理: 影検出の閾値
    pub shadow_threshold: f32,
    /// 影の処理: 影除去の強度
    pub shadow_removal_strength: f32,
    
    /// エッジシャープニング有効/無効
    pub sharpen_enabled: bool,
    /// エッジシャープニング: シャープニング強度
    pub sharpen_amount: f32,
    /// エッジシャープニング: シャープニング半径
    pub sharpen_radius: f32,
    /// エッジシャープニング: シャープニング閾値
    pub sharpen_threshold: f32,
    
    /// 適応的許容範囲有効/無効
    pub adaptive_tolerance_enabled: bool,
    /// 適応的許容範囲: グリッドサイズ（幅）
    pub adaptive_tolerance_grid_w: u32,
    /// 適応的許容範囲: グリッドサイズ（高さ）
    pub adaptive_tolerance_grid_h: u32,
    /// 適応的許容範囲: 感度調整
    pub adaptive_tolerance_sensitivity: f32,
    
    /// エッジ検出方法
    pub edge_detection_method: EdgeDetectionMethod,
    /// Cannyエッジ検出: 低閾値
    pub canny_low_threshold: f32,
    /// Cannyエッジ検出: 高閾値
    pub canny_high_threshold: f32,
    /// Cannyエッジ検出: ガウシアンシグマ
    pub canny_gaussian_sigma: f32,
    
    /// デスピル方法
    pub despill_method: DespillMethod,
    
    /// 細線検出有効/無効
    pub thin_line_detection_enabled: bool,
    /// 細線検出: 感度
    pub thin_line_sensitivity: f32,
    /// 細線検出: 検出閾値
    pub thin_line_threshold: f32,
    
    /// 自動パラメータ推定有効/無効
    pub auto_params_enabled: bool,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            chroma_color: Rgb::new(0, 255, 0), // 緑
            tolerance: 0.3,
            feather_amount: 5,
            despill_strength: 0.7,
            erode_iterations: 0,
            dilate_iterations: 1,
            verbose: false,
            
            // 新規フィールドのデフォルト値
            multi_colors: None,
            color_space: ColorSpace::Hsv,
            
            bilateral_enabled: false,
            bilateral_spatial_sigma: 5.0,
            bilateral_color_sigma: 50.0,
            bilateral_radius: 5,
            
            multiscale_enabled: false,
            multiscale_levels: 3,
            multiscale_scale_factor: 0.5,
            
            edge_optimization_enabled: false,
            edge_threshold: 0.1,
            edge_smoothness: 0.5,
            
            shadow_removal_enabled: false,
            shadow_threshold: 0.3,
            shadow_removal_strength: 0.7,
            
            sharpen_enabled: false,
            sharpen_amount: 0.5,
            sharpen_radius: 1.0,
            sharpen_threshold: 0.0,
            
            adaptive_tolerance_enabled: false,
            adaptive_tolerance_grid_w: 8,
            adaptive_tolerance_grid_h: 8,
            adaptive_tolerance_sensitivity: 1.0,
            
            edge_detection_method: EdgeDetectionMethod::Sobel,
            canny_low_threshold: 0.1,
            canny_high_threshold: 0.3,
            canny_gaussian_sigma: 1.0,
            
            despill_method: DespillMethod::Basic,
            
            thin_line_detection_enabled: false,
            thin_line_sensitivity: 0.5,
            thin_line_threshold: 0.3,
            
            auto_params_enabled: false,
        }
    }
}

impl ProcessConfig {
    /// パラメータのバリデーション
    pub fn validate(&self) -> Result<()> {
        // tolerance: 0.0 - 1.0
        if !(0.0..=1.0).contains(&self.tolerance) {
            return Err(ChromaError::ParameterOutOfRange {
                name: "tolerance".to_string(),
                value: self.tolerance,
                min: 0.0,
                max: 1.0,
            });
        }

        // feather_amount: 0 - 50
        if self.feather_amount > 50 {
            return Err(ChromaError::ParameterOutOfRange {
                name: "feather".to_string(),
                value: self.feather_amount as f32,
                min: 0.0,
                max: 50.0,
            });
        }

        // despill_strength: 0.0 - 1.0
        if !(0.0..=1.0).contains(&self.despill_strength) {
            return Err(ChromaError::ParameterOutOfRange {
                name: "despill".to_string(),
                value: self.despill_strength,
                min: 0.0,
                max: 1.0,
            });
        }

        // erode_iterations: 0 - 10
        if self.erode_iterations > 10 {
            return Err(ChromaError::ParameterOutOfRange {
                name: "erode".to_string(),
                value: self.erode_iterations as f32,
                min: 0.0,
                max: 10.0,
            });
        }

        // dilate_iterations: 0 - 10
        if self.dilate_iterations > 10 {
            return Err(ChromaError::ParameterOutOfRange {
                name: "dilate".to_string(),
                value: self.dilate_iterations as f32,
                min: 0.0,
                max: 10.0,
            });
        }

        Ok(())
    }

    /// CLIからConfigを構築
    #[cfg(feature = "cli")]
    pub fn from_cli(args: &Args) -> Result<Self> {
        // HEXコードまたは色名を受け付ける
        let chroma_color = Rgb::from_color_spec(&args.color)?;
        
        // 多色検出の設定
        let multi_colors = if args.multi_color.is_empty() {
            None
        } else {
            Some(
                args.multi_color
                    .iter()
                    .map(|s| {
                        let parts: Vec<&str> = s.split(':').collect();
                        if parts.len() != 2 {
                            return Err(ChromaError::InvalidParameter {
                                name: "multi-color".to_string(),
                                value: s.clone(),
                            });
                        }
                        let color = Rgb::from_color_spec(parts[0])?;
                        let tolerance = parts[1]
                            .parse::<f32>()
                            .map_err(|_| ChromaError::InvalidParameter {
                                name: "multi-color tolerance".to_string(),
                                value: parts[1].to_string(),
                            })?;
                        Ok(ColorConfig { color, tolerance })
                    })
                    .collect::<Result<Vec<_>>>()?,
            )
        };
        
        // 色空間の設定
        let color_space = ColorSpace::from_str(&args.color_space)
            .map_err(|e| ChromaError::InvalidParameter {
                name: "color-space".to_string(),
                value: e,
            })?;

        Ok(Self {
            chroma_color,
            tolerance: args.tolerance,
            feather_amount: args.feather,
            despill_strength: args.despill,
            erode_iterations: args.erode,
            dilate_iterations: args.dilate,
            verbose: args.verbose > 0,
            
            multi_colors,
            color_space,
            
            bilateral_enabled: args.bilateral,
            bilateral_spatial_sigma: args.bilateral_spatial_sigma,
            bilateral_color_sigma: args.bilateral_color_sigma,
            bilateral_radius: args.bilateral_radius,
            
            multiscale_enabled: args.multiscale,
            multiscale_levels: args.multiscale_levels,
            multiscale_scale_factor: args.multiscale_scale_factor,
            
            edge_optimization_enabled: args.edge_optimization,
            edge_threshold: args.edge_threshold,
            edge_smoothness: args.edge_smoothness,
            
            shadow_removal_enabled: args.shadow_removal,
            shadow_threshold: args.shadow_threshold,
            shadow_removal_strength: args.shadow_removal_strength,
            
            sharpen_enabled: args.sharpen,
            sharpen_amount: args.sharpen_amount,
            sharpen_radius: args.sharpen_radius,
            sharpen_threshold: args.sharpen_threshold,
            
            adaptive_tolerance_enabled: args.adaptive_tolerance,
            adaptive_tolerance_grid_w: args.adaptive_tolerance_grid_w,
            adaptive_tolerance_grid_h: args.adaptive_tolerance_grid_h,
            adaptive_tolerance_sensitivity: args.adaptive_tolerance_sensitivity,
            
            edge_detection_method: match args.edge_detection_method.to_lowercase().as_str() {
                "sobel" => EdgeDetectionMethod::Sobel,
                "canny" => EdgeDetectionMethod::Canny,
                _ => return Err(ChromaError::InvalidParameter {
                    name: "edge-detection-method".to_string(),
                    value: args.edge_detection_method.clone(),
                }),
            },
            canny_low_threshold: args.canny_low_threshold,
            canny_high_threshold: args.canny_high_threshold,
            canny_gaussian_sigma: args.canny_gaussian_sigma,
            
            despill_method: match args.despill_method.to_lowercase().as_str() {
                "basic" => DespillMethod::Basic,
                "advanced" => DespillMethod::Advanced,
                _ => return Err(ChromaError::InvalidParameter {
                    name: "despill-method".to_string(),
                    value: args.despill_method.clone(),
                }),
            },
            
            thin_line_detection_enabled: args.thin_line_detection,
            thin_line_sensitivity: args.thin_line_sensitivity,
            thin_line_threshold: args.thin_line_threshold,
            
            auto_params_enabled: args.auto_params,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ProcessConfig::default();
        assert_eq!(config.chroma_color, Rgb::new(0, 255, 0));
        assert!((config.tolerance - 0.3).abs() < 0.001);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_tolerance_out_of_range() {
        let mut config = ProcessConfig::default();
        config.tolerance = 1.5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_feather_out_of_range() {
        let mut config = ProcessConfig::default();
        config.feather_amount = 100;
        assert!(config.validate().is_err());
    }
}

