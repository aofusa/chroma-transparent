//! 色処理モジュール

mod convert;
mod hsv;
pub mod lut;
pub mod names;
mod rgb;

pub use convert::{hsv_to_rgb, rgb_to_hsv, rgb_to_lab, rgb_to_lch, rgb_to_yuv, Lab, Lch, Yuv};
pub use hsv::Hsv;
pub use lut::{rgb_distance_normalized, rgb_distance_squared, HsvLut};
pub use names::{all_color_names, find_color_by_name};
pub use rgb::Rgb;

/// 色空間の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    Hsv,
    Lab,
    Lch,
    Yuv,
}

impl ColorSpace {
    /// 文字列から色空間を取得
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "hsv" => Ok(ColorSpace::Hsv),
            "lab" => Ok(ColorSpace::Lab),
            "lch" => Ok(ColorSpace::Lch),
            "yuv" => Ok(ColorSpace::Yuv),
            _ => Err(format!("Unknown color space: {}", s)),
        }
    }

    /// 色空間名を文字列で取得
    pub fn as_str(&self) -> &'static str {
        match self {
            ColorSpace::Hsv => "hsv",
            ColorSpace::Lab => "lab",
            ColorSpace::Lch => "lch",
            ColorSpace::Yuv => "yuv",
        }
    }
}

impl Default for ColorSpace {
    fn default() -> Self {
        ColorSpace::Hsv
    }
}

