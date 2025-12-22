//! カスタムエラー型

use thiserror::Error;

/// クロマキー処理で発生するエラー
#[derive(Error, Debug)]
pub enum ChromaError {
    /// 画像の読み込みに失敗
    #[error("Failed to load image: {path}")]
    ImageLoadError {
        path: String,
        #[source]
        source: image::ImageError,
    },

    /// 画像の保存に失敗
    #[error("Failed to save image: {path}")]
    ImageSaveError {
        path: String,
        #[source]
        source: image::ImageError,
    },

    /// 無効なHEXカラーコード
    #[error("Invalid HEX color code: {hex}")]
    InvalidHexColor { hex: String },

    /// パラメータが範囲外
    #[error("Parameter out of range: {name} = {value} (range: {min} - {max})")]
    ParameterOutOfRange {
        name: String,
        value: f32,
        min: f32,
        max: f32,
    },

    /// 入力ファイルが存在しない
    #[error("Input file not found: {path}")]
    InputFileNotFound { path: String },
}

/// Result型のエイリアス
pub type Result<T> = std::result::Result<T, ChromaError>;

