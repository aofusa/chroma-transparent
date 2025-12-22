//! RGB色構造体

use crate::error::{ChromaError, Result};

/// RGB色構造体
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    /// 新しいRGB色を作成
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// HEXコード文字列からRGBを生成
    /// 対応形式: "RRGGBB", "#RRGGBB", "RGB", "#RGB"
    pub fn from_hex(hex: &str) -> Result<Self> {
        let hex = hex.trim_start_matches('#');

        match hex.len() {
            // 短縮形式 (RGB)
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16)
                    .map_err(|_| ChromaError::InvalidHexColor { hex: hex.to_string() })?;
                let g = u8::from_str_radix(&hex[1..2], 16)
                    .map_err(|_| ChromaError::InvalidHexColor { hex: hex.to_string() })?;
                let b = u8::from_str_radix(&hex[2..3], 16)
                    .map_err(|_| ChromaError::InvalidHexColor { hex: hex.to_string() })?;
                // 短縮形式は各桁を2回繰り返す (F -> FF)
                Ok(Self::new(r * 17, g * 17, b * 17))
            }
            // 通常形式 (RRGGBB)
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| ChromaError::InvalidHexColor { hex: hex.to_string() })?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| ChromaError::InvalidHexColor { hex: hex.to_string() })?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| ChromaError::InvalidHexColor { hex: hex.to_string() })?;
                Ok(Self::new(r, g, b))
            }
            _ => Err(ChromaError::InvalidHexColor { hex: hex.to_string() }),
        }
    }

    /// HEXコード文字列に変換
    pub fn to_hex(&self) -> String {
        format!("{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// image crateのRgba型から変換
    pub fn from_rgba(rgba: image::Rgba<u8>) -> Self {
        Self::new(rgba[0], rgba[1], rgba[2])
    }

    /// 正規化されたRGB値を取得 (0.0 - 1.0)
    pub fn normalized(&self) -> (f32, f32, f32) {
        (
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        )
    }
}

impl Default for Rgb {
    fn default() -> Self {
        Self::new(0, 255, 0) // 緑
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hex_6digit() {
        let rgb = Rgb::from_hex("00FF00").unwrap();
        assert_eq!(rgb, Rgb::new(0, 255, 0));
    }

    #[test]
    fn test_from_hex_with_hash() {
        let rgb = Rgb::from_hex("#FF0000").unwrap();
        assert_eq!(rgb, Rgb::new(255, 0, 0));
    }

    #[test]
    fn test_from_hex_3digit() {
        let rgb = Rgb::from_hex("F00").unwrap();
        assert_eq!(rgb, Rgb::new(255, 0, 0));
    }

    #[test]
    fn test_to_hex() {
        let rgb = Rgb::new(255, 128, 0);
        assert_eq!(rgb.to_hex(), "FF8000");
    }

    #[test]
    fn test_normalized() {
        let rgb = Rgb::new(255, 128, 0);
        let (r, g, b) = rgb.normalized();
        assert!((r - 1.0).abs() < 0.001);
        assert!((g - 0.502).abs() < 0.01);
        assert!((b - 0.0).abs() < 0.001);
    }
}

