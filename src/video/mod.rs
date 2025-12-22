//! 動画処理モジュール

mod detector;
mod ffmpeg;
pub mod processor;

pub use detector::{detect_file_type, is_video_file, FileType};
pub use ffmpeg::{Ffmpeg, VideoEncodeConfig, VideoFormat, VideoInfo};
pub use processor::{generate_video_output_path, VideoProcessConfig, VideoProcessor};

