//! 動画処理パイプライン

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use tempfile::TempDir;

use crate::pipeline::ChromaPipeline;
use crate::video::ffmpeg::{Ffmpeg, VideoEncodeConfig, VideoFormat};

/// 動画処理設定
#[derive(Debug, Clone)]
pub struct VideoProcessConfig {
    /// 出力フォーマット
    pub format: VideoFormat,
    /// 出力品質 (1-100)
    pub quality: u32,
    /// 出力フレームレート（Noneの場合は入力と同じ）
    pub fps: Option<f32>,
    /// 詳細ログ
    pub verbose: bool,
}

impl Default for VideoProcessConfig {
    fn default() -> Self {
        Self {
            format: VideoFormat::WebM,
            quality: 80,
            fps: None,
            verbose: false,
        }
    }
}

/// 動画処理パイプライン
pub struct VideoProcessor {
    ffmpeg: Ffmpeg,
    chroma_pipeline: ChromaPipeline,
    config: VideoProcessConfig,
}

impl VideoProcessor {
    /// 新しい動画処理パイプラインを作成
    pub fn new(
        ffmpeg: Ffmpeg,
        chroma_pipeline: ChromaPipeline,
        config: VideoProcessConfig,
    ) -> Self {
        Self {
            ffmpeg,
            chroma_pipeline,
            config,
        }
    }

    /// 動画を処理
    pub fn process(&self, input: &Path, output: &Path) -> Result<()> {
        // 1. ffmpegが利用可能か確認
        if !self.ffmpeg.is_available() {
            return Err(anyhow!(
                "ffmpeg not found. Please install ffmpeg and ensure it's in your PATH."
            ));
        }

        self.log("Processing video...");

        // 2. 動画情報を取得
        let info = self.ffmpeg.probe(input)
            .context("Failed to probe video")?;
        
        self.log(&format!(
            "Video info: {}x{}, {:.2}fps, {:.2}s, ~{} frames",
            info.width, info.height, info.fps, info.duration, info.frame_count
        ));

        // 3. 一時ディレクトリを作成
        let temp_dir = TempDir::new()
            .context("Failed to create temp directory")?;
        
        let input_frames_dir = temp_dir.path().join("input");
        let output_frames_dir = temp_dir.path().join("output");
        
        fs::create_dir_all(&input_frames_dir)?;
        fs::create_dir_all(&output_frames_dir)?;

        // 4. フレームを抽出
        self.log("Extracting frames...");
        let frame_count = self.ffmpeg.extract_frames(
            input,
            &input_frames_dir,
            self.config.fps,
        ).context("Failed to extract frames")?;
        
        self.log(&format!("Extracted {} frames", frame_count));

        if frame_count == 0 {
            return Err(anyhow!("No frames extracted from video"));
        }

        // 5. 各フレームにクロマキー処理を適用
        self.log("Processing frames...");
        self.process_frames(&input_frames_dir, &output_frames_dir, frame_count)?;

        // 6. PNG連番出力の場合はここで終了
        if self.config.format == VideoFormat::PngSequence {
            // 出力ディレクトリにコピー
            self.copy_frames_to_output(&output_frames_dir, output)?;
            return Ok(());
        }

        // 7. 処理済みフレームを動画にエンコード
        self.log("Encoding video...");
        let fps = self.config.fps.unwrap_or(info.fps);
        let encode_config = VideoEncodeConfig {
            format: self.config.format,
            fps,
            quality: self.config.quality,
            audio_input: if info.has_audio {
                Some(input.to_path_buf())
            } else {
                None
            },
        };

        let input_pattern = output_frames_dir.join("%06d.png");
        self.ffmpeg.encode_video(&input_pattern, output, &encode_config)
            .context("Failed to encode video")?;

        self.log("Video processing completed");

        // 一時ディレクトリは自動削除される
        Ok(())
    }

    /// フレームを処理
    fn process_frames(
        &self,
        input_dir: &Path,
        output_dir: &Path,
        frame_count: u32,
    ) -> Result<()> {
        let mut processed = 0;

        for i in 1..=frame_count {
            let input_path = input_dir.join(format!("{:06}.png", i));
            let output_path = output_dir.join(format!("{:06}.png", i));

            if !input_path.exists() {
                continue;
            }

            // 画像を読み込み
            let image = image::open(&input_path)
                .with_context(|| format!("Failed to open frame: {:?}", input_path))?
                .to_rgba8();

            // クロマキー処理
            let result = self.chroma_pipeline.process(&image);

            // 保存
            result.save(&output_path)
                .with_context(|| format!("Failed to save frame: {:?}", output_path))?;

            processed += 1;

            // 進捗表示（10%ごと）
            if self.config.verbose && processed % (frame_count / 10).max(1) == 0 {
                let percent = (processed as f32 / frame_count as f32 * 100.0) as u32;
                eprintln!("[chroma-transparent] Progress: {}% ({}/{})", percent, processed, frame_count);
            }
        }

        self.log(&format!("Processed {} frames", processed));
        Ok(())
    }

    /// PNG連番を出力ディレクトリにコピー
    fn copy_frames_to_output(&self, frames_dir: &Path, output: &Path) -> Result<()> {
        // 出力がディレクトリパスかどうか判定
        let output_dir = if output.extension().is_none() 
            || output.to_string_lossy().ends_with('/') 
            || output.to_string_lossy().ends_with('\\') 
        {
            output.to_path_buf()
        } else {
            // ファイル名が指定されている場合は親ディレクトリ + パターンとして扱う
            output.parent().unwrap_or(Path::new(".")).to_path_buf()
        };

        fs::create_dir_all(&output_dir)?;

        // フレームをコピー
        for entry in fs::read_dir(frames_dir)? {
            let entry = entry?;
            let src = entry.path();
            if src.extension().map(|e| e == "png").unwrap_or(false) {
                let filename = src.file_name().unwrap();
                let dst = output_dir.join(filename);
                fs::copy(&src, &dst)?;
            }
        }

        self.log(&format!("Frames saved to: {:?}", output_dir));
        Ok(())
    }

    /// ログ出力
    fn log(&self, message: &str) {
        if self.config.verbose {
            eprintln!("[chroma-transparent] {}", message);
        }
    }
}

/// 動画の出力パスを生成
pub fn generate_video_output_path(input: &Path, format: VideoFormat) -> PathBuf {
    let stem = input.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let parent = input.parent().unwrap_or(Path::new("."));
    
    match format {
        VideoFormat::PngSequence => parent.join(format!("{}_frames", stem)),
        _ => parent.join(format!("{}.chroma.{}", stem, format.extension())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_video_output_path_webm() {
        let input = Path::new("/path/to/video.mp4");
        let output = generate_video_output_path(input, VideoFormat::WebM);
        assert_eq!(output, PathBuf::from("/path/to/video.chroma.webm"));
    }

    #[test]
    fn test_generate_video_output_path_mov() {
        let input = Path::new("/path/to/video.mp4");
        let output = generate_video_output_path(input, VideoFormat::Mov);
        assert_eq!(output, PathBuf::from("/path/to/video.chroma.mov"));
    }

    #[test]
    fn test_generate_video_output_path_png_sequence() {
        let input = Path::new("/path/to/video.mp4");
        let output = generate_video_output_path(input, VideoFormat::PngSequence);
        assert_eq!(output, PathBuf::from("/path/to/video_frames"));
    }
}

