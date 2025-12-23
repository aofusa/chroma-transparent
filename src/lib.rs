//! # chroma-transparent
//!
//! 指定した色をクロマキー処理して透過PNGに変換するライブラリ
//!
//! ## 使用例
//!
//! ```rust,ignore
//! use chroma_transparent::{ChromaPipeline, ProcessConfig, Rgb};
//! use image::open;
//!
//! let config = ProcessConfig {
//!     chroma_color: Rgb::from_hex("00FF00").unwrap(),
//!     tolerance: 0.3,
//!     ..Default::default()
//! };
//!
//! let pipeline = ChromaPipeline::new(config);
//! let image = open("input.png").unwrap().to_rgba8();
//! let result = pipeline.process(&image);
//! result.save("output.png").unwrap();
//! ```

// コアモジュール（すべての環境で利用可能）
pub mod color;
pub mod config;
pub mod error;
pub mod pipeline;
pub mod processor;

// CLI専用モジュール（WASM以外）
#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "cli")]
pub mod scanner;

// 動画機能
#[cfg(feature = "video")]
pub mod video;

// サーバ機能
#[cfg(feature = "server")]
pub mod server;

// WASM機能
#[cfg(feature = "wasm")]
pub mod wasm;

// 主要な型を再エクスポート
pub use color::{ColorSpace, Rgb};
pub use config::{ColorConfig, ProcessConfig};
pub use error::{ChromaError, Result};
pub use pipeline::ChromaPipeline;

#[cfg(feature = "cli")]
pub use cli::Args;

#[cfg(feature = "cli")]
pub use scanner::{ensure_output_directory, ImageScanner};

#[cfg(feature = "video")]
pub use video::{
    detect_file_type, generate_video_output_path, FileType, Ffmpeg, VideoFormat,
    VideoProcessConfig, VideoProcessor,
};

#[cfg(feature = "server")]
pub use server::{ServerConfig, StorageManager};

// WASMエクスポート（wasm feature有効時）
#[cfg(feature = "wasm")]
pub use wasm::*;

