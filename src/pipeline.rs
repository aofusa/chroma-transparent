//! 処理パイプライン

use image::RgbaImage;
use log::debug;

use crate::config::ProcessConfig;
use crate::processor::{
    apply_alpha, create_alpha_from_mask, create_chroma_mask, despill, dilate, erode, feather_alpha,
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

    /// 画像を処理
    pub fn process(&self, image: &RgbaImage) -> RgbaImage {
        debug!("Processing started...");

        // 1. クロマキーマスク生成
        let mask = create_chroma_mask(image, &self.config.chroma_color, self.config.tolerance);
        debug!("Mask generation completed");

        // 2. モルフォロジー演算
        let mask = if self.config.erode_iterations > 0 {
            let result = erode(&mask, self.config.erode_iterations);
            debug!("Erode completed ({} iterations)", self.config.erode_iterations);
            result
        } else {
            mask
        };

        let mask = if self.config.dilate_iterations > 0 {
            let result = dilate(&mask, self.config.dilate_iterations);
            debug!("Dilate completed ({} iterations)", self.config.dilate_iterations);
            result
        } else {
            mask
        };

        // 3. アルファチャンネル生成
        let alpha_channel = create_alpha_from_mask(&mask);
        debug!("Alpha channel generation completed");

        // 4. フェザリング
        let alpha_channel = if self.config.feather_amount > 0 {
            let result = feather_alpha(&alpha_channel, self.config.feather_amount);
            debug!(
                "Feathering completed (amount={})",
                self.config.feather_amount
            );
            result
        } else {
            alpha_channel
        };

        // 5. デスピル処理
        let mut result = image.clone();
        if self.config.despill_strength > 0.0 {
            despill(
                &mut result,
                &self.config.chroma_color,
                self.config.despill_strength,
            );
            debug!(
                "Despill completed (strength={})",
                self.config.despill_strength
            );
        }

        // 6. アルファチャンネル適用
        apply_alpha(&mut result, &alpha_channel);
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
        };

        let pipeline = ChromaPipeline::new(config);
        let image = create_test_image();
        let result = pipeline.process(&image);

        // 処理が正常に完了すること
        assert!(result.width() == 10);
        assert!(result.height() == 10);
    }
}

