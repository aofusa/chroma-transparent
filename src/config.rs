//! 処理パラメータ設定

use crate::cli::Args;
use crate::color::Rgb;
use crate::error::{ChromaError, Result};

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
    pub fn from_cli(args: &Args) -> Result<Self> {
        // HEXコードまたは色名を受け付ける
        let chroma_color = Rgb::from_color_spec(&args.color)?;

        Ok(Self {
            chroma_color,
            tolerance: args.tolerance,
            feather_amount: args.feather,
            despill_strength: args.despill,
            erode_iterations: args.erode,
            dilate_iterations: args.dilate,
            verbose: args.verbose,
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

