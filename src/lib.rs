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

pub mod cli;
pub mod color;
pub mod config;
pub mod error;
pub mod pipeline;
pub mod processor;
pub mod scanner;

// 主要な型を再エクスポート
pub use cli::Args;
pub use color::Rgb;
pub use config::ProcessConfig;
pub use error::{ChromaError, Result};
pub use pipeline::ChromaPipeline;
pub use scanner::{ensure_output_directory, ImageScanner};

