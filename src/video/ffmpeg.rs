//! ffmpegラッパーモジュール

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context, Result};

/// 動画出力フォーマット
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VideoFormat {
    /// WebM (VP9 with alpha) - Web対応、推奨
    #[default]
    WebM,
    /// MOV (ProRes 4444) - 高品質、プロ用
    Mov,
    /// PNG連番
    PngSequence,
}

impl VideoFormat {
    /// 拡張子を取得
    pub fn extension(&self) -> &'static str {
        match self {
            VideoFormat::WebM => "webm",
            VideoFormat::Mov => "mov",
            VideoFormat::PngSequence => "png",
        }
    }

    /// 文字列からパース
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "webm" => Some(VideoFormat::WebM),
            "mov" => Some(VideoFormat::Mov),
            "png" | "png-sequence" => Some(VideoFormat::PngSequence),
            _ => None,
        }
    }
}

impl std::fmt::Display for VideoFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoFormat::WebM => write!(f, "webm"),
            VideoFormat::Mov => write!(f, "mov"),
            VideoFormat::PngSequence => write!(f, "png-sequence"),
        }
    }
}

/// 動画情報
#[derive(Debug, Clone)]
pub struct VideoInfo {
    /// 幅
    pub width: u32,
    /// 高さ
    pub height: u32,
    /// フレームレート
    pub fps: f32,
    /// 長さ（秒）
    pub duration: f32,
    /// フレーム数（推定）
    pub frame_count: u32,
    /// 音声があるか
    pub has_audio: bool,
}

/// エンコード設定
#[derive(Debug, Clone)]
pub struct VideoEncodeConfig {
    /// 出力フォーマット
    pub format: VideoFormat,
    /// フレームレート
    pub fps: f32,
    /// 品質 (1-100)
    pub quality: u32,
    /// 音声入力（元動画のパス）
    pub audio_input: Option<PathBuf>,
}

/// ffmpegラッパー
pub struct Ffmpeg {
    /// ffmpegのパス
    path: PathBuf,
    /// ffprobeのパス
    probe_path: PathBuf,
    /// 詳細ログ
    verbose: bool,
}

impl Ffmpeg {
    /// 新しいFfmpegインスタンスを作成
    pub fn new(path: &Path) -> Self {
        let path = path.to_path_buf();
        // ffprobeはffmpegと同じディレクトリにあると仮定
        let probe_path = if let Some(parent) = path.parent() {
            if parent.as_os_str().is_empty() {
                PathBuf::from("ffprobe")
            } else {
                parent.join("ffprobe")
            }
        } else {
            PathBuf::from("ffprobe")
        };

        Self {
            path,
            probe_path,
            verbose: false,
        }
    }

    /// 詳細ログを有効化
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// ffmpegが利用可能か確認
    pub fn is_available(&self) -> bool {
        Command::new(&self.path)
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// ffmpegのバージョンを取得
    pub fn version(&self) -> Result<String> {
        let output = Command::new(&self.path)
            .arg("-version")
            .output()
            .context("Failed to execute ffmpeg")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let first_line = stdout.lines().next().unwrap_or("unknown");
        Ok(first_line.to_string())
    }

    /// 動画の情報を取得
    pub fn probe(&self, input: &Path) -> Result<VideoInfo> {
        // ffprobeでJSON出力を取得
        let output = Command::new(&self.probe_path)
            .args([
                "-v", "quiet",
                "-print_format", "json",
                "-show_format",
                "-show_streams",
            ])
            .arg(input)
            .output()
            .context("Failed to execute ffprobe")?;

        if !output.status.success() {
            return Err(anyhow!(
                "ffprobe failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)
            .context("Failed to parse ffprobe output")?;

        // ビデオストリームを探す
        let streams = json["streams"].as_array()
            .ok_or_else(|| anyhow!("No streams found"))?;

        let video_stream = streams
            .iter()
            .find(|s| s["codec_type"].as_str() == Some("video"))
            .ok_or_else(|| anyhow!("No video stream found"))?;

        let audio_exists = streams
            .iter()
            .any(|s| s["codec_type"].as_str() == Some("audio"));

        // 幅と高さ
        let width = video_stream["width"].as_u64().unwrap_or(0) as u32;
        let height = video_stream["height"].as_u64().unwrap_or(0) as u32;

        // フレームレート (例: "30/1" or "30000/1001")
        let fps_str = video_stream["r_frame_rate"].as_str().unwrap_or("30/1");
        let fps = parse_frame_rate(fps_str);

        // 長さ
        let duration = json["format"]["duration"]
            .as_str()
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0);

        // フレーム数（推定）
        let frame_count = (duration * fps).ceil() as u32;

        Ok(VideoInfo {
            width,
            height,
            fps,
            duration,
            frame_count,
            has_audio: audio_exists,
        })
    }

    /// フレームを抽出（PNG連番として出力）
    pub fn extract_frames(
        &self,
        input: &Path,
        output_dir: &Path,
        fps: Option<f32>,
    ) -> Result<u32> {
        std::fs::create_dir_all(output_dir)
            .context("Failed to create output directory")?;

        let output_pattern = output_dir.join("%06d.png");

        let mut cmd = Command::new(&self.path);
        cmd.args(["-i"]);
        cmd.arg(input);

        // フレームレート指定
        if let Some(fps) = fps {
            cmd.args(["-r", &fps.to_string()]);
        }

        // PNG出力
        cmd.args(["-f", "image2"]);
        cmd.arg(&output_pattern);

        // 上書き許可
        cmd.arg("-y");

        if self.verbose {
            eprintln!("[ffmpeg] Extracting frames...");
        } else {
            cmd.stdout(Stdio::null());
            cmd.stderr(Stdio::null());
        }

        let status = cmd.status().context("Failed to execute ffmpeg")?;

        if !status.success() {
            return Err(anyhow!("ffmpeg frame extraction failed"));
        }

        // 抽出されたフレーム数をカウント
        let count = std::fs::read_dir(output_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "png").unwrap_or(false))
            .count() as u32;

        Ok(count)
    }

    /// フレームから動画をエンコード
    pub fn encode_video(
        &self,
        input_pattern: &Path,
        output: &Path,
        config: &VideoEncodeConfig,
    ) -> Result<()> {
        let mut cmd = Command::new(&self.path);

        // 入力フレームレート
        cmd.args(["-framerate", &config.fps.to_string()]);

        // 入力パターン
        cmd.args(["-i"]);
        cmd.arg(input_pattern);

        // 音声入力（あれば）
        if let Some(ref audio_input) = config.audio_input {
            if config.format != VideoFormat::PngSequence {
                cmd.args(["-i"]);
                cmd.arg(audio_input);
                cmd.args(["-map", "0:v", "-map", "1:a?", "-shortest"]);
            }
        }

        // フォーマット別エンコード設定
        match config.format {
            VideoFormat::WebM => {
                // VP9 with alpha channel
                cmd.args([
                    "-c:v", "libvpx-vp9",
                    "-pix_fmt", "yuva420p",
                    "-auto-alt-ref", "0",
                ]);
                // 品質設定 (CRF: 0-63, lower is better)
                let crf = 63 - (config.quality as i32 * 63 / 100);
                cmd.args(["-crf", &crf.to_string()]);
                cmd.args(["-b:v", "0"]);
                // 音声コーデック
                if config.audio_input.is_some() {
                    cmd.args(["-c:a", "libopus"]);
                }
            }
            VideoFormat::Mov => {
                // ProRes 4444 (with alpha)
                cmd.args([
                    "-c:v", "prores_ks",
                    "-profile:v", "4444",
                    "-pix_fmt", "yuva444p10le",
                ]);
                // 音声コーデック
                if config.audio_input.is_some() {
                    cmd.args(["-c:a", "aac"]);
                }
            }
            VideoFormat::PngSequence => {
                // PNG連番はextract_framesで処理済みなので、ここでは何もしない
                return Ok(());
            }
        }

        // 出力
        cmd.arg(output);

        // 上書き許可
        cmd.arg("-y");

        if self.verbose {
            eprintln!("[ffmpeg] Encoding video...");
        } else {
            cmd.stdout(Stdio::null());
            cmd.stderr(Stdio::null());
        }

        let status = cmd.status().context("Failed to execute ffmpeg")?;

        if !status.success() {
            return Err(anyhow!("ffmpeg encoding failed"));
        }

        Ok(())
    }
}

/// フレームレート文字列をパース (例: "30/1", "30000/1001")
fn parse_frame_rate(s: &str) -> f32 {
    if let Some((num, den)) = s.split_once('/') {
        let num: f32 = num.parse().unwrap_or(30.0);
        let den: f32 = den.parse().unwrap_or(1.0);
        if den != 0.0 {
            return num / den;
        }
    }
    s.parse().unwrap_or(30.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_format_extension() {
        assert_eq!(VideoFormat::WebM.extension(), "webm");
        assert_eq!(VideoFormat::Mov.extension(), "mov");
        assert_eq!(VideoFormat::PngSequence.extension(), "png");
    }

    #[test]
    fn test_video_format_from_str() {
        assert_eq!(VideoFormat::from_str("webm"), Some(VideoFormat::WebM));
        assert_eq!(VideoFormat::from_str("WEBM"), Some(VideoFormat::WebM));
        assert_eq!(VideoFormat::from_str("mov"), Some(VideoFormat::Mov));
        assert_eq!(VideoFormat::from_str("png"), Some(VideoFormat::PngSequence));
        assert_eq!(VideoFormat::from_str("png-sequence"), Some(VideoFormat::PngSequence));
        assert_eq!(VideoFormat::from_str("unknown"), None);
    }

    #[test]
    fn test_parse_frame_rate() {
        assert!((parse_frame_rate("30/1") - 30.0).abs() < 0.01);
        assert!((parse_frame_rate("30000/1001") - 29.97).abs() < 0.01);
        assert!((parse_frame_rate("24/1") - 24.0).abs() < 0.01);
        assert!((parse_frame_rate("60") - 60.0).abs() < 0.01);
    }
}

