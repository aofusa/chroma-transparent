//! カスタムエラー型

use thiserror::Error;

/// クロマキー処理で発生するエラー
#[derive(Error, Debug)]
pub enum ChromaError {
    /// 画像の読み込みに失敗
    #[error("画像の読み込みに失敗しました: {path}")]
    ImageLoadError {
        path: String,
        #[source]
        source: image::ImageError,
    },

    /// 画像の保存に失敗
    #[error("画像の保存に失敗しました: {path}")]
    ImageSaveError {
        path: String,
        #[source]
        source: image::ImageError,
    },

    /// 無効なHEXカラーコード
    #[error("無効なHEXカラーコード: {hex}")]
    InvalidHexColor { hex: String },

    /// パラメータが範囲外
    #[error("パラメータが範囲外です: {name} = {value} (範囲: {min} - {max})")]
    ParameterOutOfRange {
        name: String,
        value: f32,
        min: f32,
        max: f32,
    },

    /// 入力ファイルが存在しない
    #[error("入力ファイルが存在しません: {path}")]
    InputFileNotFound { path: String },
}

/// Result型のエイリアス
pub type Result<T> = std::result::Result<T, ChromaError>;

