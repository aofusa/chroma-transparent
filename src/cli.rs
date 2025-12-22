//! コマンドライン引数パーサー

use clap::Parser;
use std::path::{Path, PathBuf};

use crate::error::{ChromaError, Result};

/// 指定した色をクロマキー処理して透過PNGに変換するCLIツール
#[derive(Parser, Debug)]
#[command(name = "chroma-transparent")]
#[command(version)]
#[command(about = "指定した色をクロマキー処理して透過PNGに変換")]
#[cfg_attr(all(feature = "video", feature = "server"), command(long_about = r#"
指定した画像または動画の指定された色をクロマキー処理して透過ファイルに変換するCLIツールです。

グリーンバック画像/動画やブルーバック画像/動画など、単色背景から
被写体を切り抜いて透過PNG/WebMを生成できます。

ファイルまたはディレクトリを入力として指定できます。
動画ファイルの場合はffmpegが必要です。

--serve オプションでWebサーバとして起動し、ブラウザからGUIで操作できます。

色の指定にはHEXコード（00FF00）またはCSS色名（lime, green, blue等）が使用できます。

例:
  chroma-transparent photo.png
  chroma-transparent photo.png -o result.png -c lime
  chroma-transparent video.mp4 -o output.webm
  chroma-transparent --serve --port 8080
"#))]
#[cfg_attr(all(feature = "server", not(feature = "video")), command(long_about = r#"
指定した画像の指定された色をクロマキー処理して透過PNGに変換するCLIツールです。

グリーンバック画像やブルーバック画像など、単色背景の画像から
被写体を切り抜いて透過PNGを生成できます。

--serve オプションでWebサーバとして起動し、ブラウザからGUIで操作できます。

色の指定にはHEXコード（00FF00）またはCSS色名（lime, green, blue等）が使用できます。

例:
  chroma-transparent photo.png
  chroma-transparent photo.png -o result.png -c lime
  chroma-transparent --serve --port 8080
"#))]
#[cfg_attr(all(feature = "video", not(feature = "server")), command(long_about = r#"
指定した画像または動画の指定された色をクロマキー処理して透過ファイルに変換するCLIツールです。

グリーンバック画像/動画やブルーバック画像/動画など、単色背景から
被写体を切り抜いて透過PNG/WebMを生成できます。

ファイルまたはディレクトリを入力として指定できます。
動画ファイルの場合はffmpegが必要です。

色の指定にはHEXコード（00FF00）またはCSS色名（lime, green, blue等）が使用できます。

例:
  chroma-transparent photo.png
  chroma-transparent photo.png -o result.png -c lime
  chroma-transparent video.mp4 -o output.webm
  chroma-transparent ./input_dir -o ./output_dir -r 2
"#))]
#[cfg_attr(not(any(feature = "video", feature = "server")), command(long_about = r#"
指定した画像の指定された色をクロマキー処理して透過PNGに変換するCLIツールです。

グリーンバック画像やブルーバック画像など、単色背景の画像から
被写体を切り抜いて透過PNGを生成できます。

ファイルまたはディレクトリを入力として指定できます。

色の指定にはHEXコード（00FF00）またはCSS色名（lime, green, blue等）が使用できます。

例:
  chroma-transparent photo.png
  chroma-transparent photo.png -o result.png -c lime
  chroma-transparent photo.png -c blue -t 0.4 -f 10 -d 0.9
  chroma-transparent ./input_dir -o ./output_dir -r 2
"#))]
pub struct Args {
    /// 入力画像またはディレクトリのパス（--serve時は不要）
    #[arg(value_name = "INPUT")]
    pub input: Option<PathBuf>,

    /// 出力ファイルまたはディレクトリのパス [デフォルト: <入力ファイル名>.chroma.png]
    #[arg(short, long, value_name = "OUTPUT")]
    pub output: Option<PathBuf>,

    /// クロマキー処理する色 (HEXコードまたはCSS色名)
    #[arg(short, long, default_value = "lime", value_name = "COLOR")]
    pub color: String,

    /// 色の許容範囲 (0.0 - 1.0)
    #[arg(short, long, default_value = "0.3", value_name = "FLOAT")]
    pub tolerance: f32,

    /// フェザリング量 (0 - 50)
    #[arg(short, long, default_value = "5", value_name = "INT")]
    pub feather: u32,

    /// デスピル強度 (0.0 - 1.0)
    #[arg(short, long, default_value = "0.7", value_name = "FLOAT")]
    pub despill: f32,

    /// 収縮回数 (0 - 10)
    #[arg(short, long, default_value = "0", value_name = "INT")]
    pub erode: u32,

    /// 膨張回数 (0 - 10)
    #[arg(short = 'D', long, default_value = "1", value_name = "INT")]
    pub dilate: u32,

    /// ディレクトリを再帰的に探索する深さ (0 = 無制限)
    /// このオプションを指定しない場合、ディレクトリ直下のファイルのみ処理します
    #[arg(short, long, value_name = "DEPTH")]
    pub recursive: Option<u32>,

    // === 動画機能 (feature = "video") ===
    /// 動画出力フォーマット [webm, mov, png-sequence]
    #[cfg(feature = "video")]
    #[arg(long, value_name = "FORMAT")]
    pub video_format: Option<String>,

    /// 動画出力品質 (1-100)
    #[cfg(feature = "video")]
    #[arg(long, default_value = "80", value_name = "QUALITY")]
    pub video_quality: u32,

    /// 動画出力フレームレート（省略時は入力と同じ）
    #[cfg(feature = "video")]
    #[arg(long, value_name = "FPS")]
    pub fps: Option<f32>,

    /// ffmpegのパス
    #[cfg(feature = "video")]
    #[arg(long, default_value = "ffmpeg", value_name = "PATH")]
    pub ffmpeg: PathBuf,

    // === サーバ機能 (feature = "server") ===
    /// サーバモードで起動
    #[cfg(feature = "server")]
    #[arg(long)]
    pub serve: bool,

    /// サーバのリッスンホスト
    #[cfg(feature = "server")]
    #[arg(long, default_value = "127.0.0.1", value_name = "HOST")]
    pub host: String,

    /// サーバのリッスンポート
    #[cfg(feature = "server")]
    #[arg(long, default_value = "8080", value_name = "PORT")]
    pub port: u16,

    /// 処理結果の保存先ディレクトリ（サーバモード時、省略時は一時ディレクトリ）
    #[cfg(feature = "server")]
    #[arg(long, value_name = "DIR")]
    pub storage_dir: Option<PathBuf>,

    /// 動画処理を有効化（--features video でビルドされている場合のみ）
    #[cfg(all(feature = "server", feature = "video"))]
    #[arg(long)]
    pub enable_video: bool,

    /// 詳細ログを出力（-v: debug, -vv: trace）
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// 静音モード（エラーのみ出力）
    #[arg(short, long)]
    pub quiet: bool,
}

impl Args {
    /// サーバモードかどうか
    #[cfg(feature = "server")]
    pub fn is_server_mode(&self) -> bool {
        self.serve
    }

    #[cfg(not(feature = "server"))]
    pub fn is_server_mode(&self) -> bool {
        false
    }

    /// 入力パスを取得（必須の場合）
    pub fn input_path(&self) -> Result<&PathBuf> {
        self.input.as_ref().ok_or_else(|| ChromaError::InputFileNotFound {
            path: "(no input specified)".to_string(),
        })
    }

    /// 入力がディレクトリかどうか
    pub fn is_input_directory(&self) -> bool {
        self.input.as_ref().map(|p| p.is_dir()).unwrap_or(false)
    }

    /// 出力がディレクトリかどうか（存在するディレクトリまたは末尾が/で終わる場合）
    pub fn is_output_directory(&self) -> bool {
        if let Some(output) = &self.output {
            output.is_dir()
                || output.to_string_lossy().ends_with('/')
                || output.to_string_lossy().ends_with('\\')
        } else {
            false
        }
    }

    /// 単一ファイル用の出力パスを取得
    pub fn output_path_for_file(&self, input_file: &Path) -> PathBuf {
        if let Some(output) = &self.output {
            if output.is_dir()
                || output.to_string_lossy().ends_with('/')
                || output.to_string_lossy().ends_with('\\')
            {
                // 出力がディレクトリの場合、デフォルト名で出力
                let stem = input_file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output");
                output.join(format!("{}.chroma.png", stem))
            } else {
                // 出力がファイルパスの場合、そのまま使用
                output.clone()
            }
        } else {
            // 出力が指定されていない場合、入力ファイルと同じディレクトリにデフォルト名で出力
            let stem = input_file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            let parent = input_file.parent().unwrap_or(Path::new("."));
            parent.join(format!("{}.chroma.png", stem))
        }
    }

    /// ディレクトリ処理用の出力パスを取得（相対パスを維持）
    pub fn output_path_for_dir_entry(
        &self,
        input_file: &Path,
        base_input_dir: &Path,
    ) -> PathBuf {
        let output_dir = self.output.clone().unwrap_or_else(|| base_input_dir.to_path_buf());

        // 入力ファイルの相対パスを取得
        let relative_path = input_file
            .strip_prefix(base_input_dir)
            .unwrap_or(input_file);

        // 出力ファイル名を生成
        let stem = input_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let output_filename = format!("{}.chroma.png", stem);

        // 相対パスの親ディレクトリを取得
        if let Some(relative_parent) = relative_path.parent() {
            if relative_parent.as_os_str().is_empty() {
                output_dir.join(output_filename)
            } else {
                output_dir.join(relative_parent).join(output_filename)
            }
        } else {
            output_dir.join(output_filename)
        }
    }

    /// 出力パスを取得（後方互換性のため維持）
    pub fn output_path(&self) -> PathBuf {
        if let Some(input) = &self.input {
            self.output_path_for_file(input)
        } else {
            PathBuf::from("output.chroma.png")
        }
    }

    /// 入力パスの存在確認
    pub fn validate_input(&self) -> Result<()> {
        if let Some(input) = &self.input {
            if !input.exists() {
                return Err(ChromaError::InputFileNotFound {
                    path: input.display().to_string(),
                });
            }
        }
        Ok(())
    }

    /// サーバ設定を構築
    #[cfg(feature = "server")]
    pub fn build_server_config(&self) -> crate::ServerConfig {
        crate::ServerConfig {
            host: self.host.clone(),
            port: self.port,
            storage_dir: self.storage_dir.clone(),
            video_enabled: {
                #[cfg(feature = "video")]
                {
                    self.enable_video
                }
                #[cfg(not(feature = "video"))]
                {
                    false
                }
            },
            cors_origin: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_args() -> Args {
        Args {
            input: Some(PathBuf::from("/path/to/image.png")),
            output: None,
            color: "00FF00".to_string(),
            tolerance: 0.3,
            feather: 5,
            despill: 0.7,
            erode: 0,
            dilate: 1,
            recursive: None,
            #[cfg(feature = "video")]
            video_format: None,
            #[cfg(feature = "video")]
            video_quality: 80,
            #[cfg(feature = "video")]
            fps: None,
            #[cfg(feature = "video")]
            ffmpeg: PathBuf::from("ffmpeg"),
            #[cfg(feature = "server")]
            serve: false,
            #[cfg(feature = "server")]
            host: "127.0.0.1".to_string(),
            #[cfg(feature = "server")]
            port: 8080,
            #[cfg(feature = "server")]
            storage_dir: None,
            #[cfg(all(feature = "server", feature = "video"))]
            enable_video: false,
            verbose: 0,
            quiet: false,
        }
    }

    #[test]
    fn test_output_path_default() {
        let args = create_test_args();
        assert_eq!(
            args.output_path(),
            PathBuf::from("/path/to/image.chroma.png")
        );
    }

    #[test]
    fn test_output_path_specified() {
        let mut args = create_test_args();
        args.output = Some(PathBuf::from("/other/output.png"));
        assert_eq!(args.output_path(), PathBuf::from("/other/output.png"));
    }

    #[test]
    fn test_output_path_for_dir_entry() {
        let mut args = create_test_args();
        args.input = Some(PathBuf::from("/input"));
        args.output = Some(PathBuf::from("/output"));
        args.recursive = Some(0);

        // サブディレクトリ内のファイル
        let input_file = Path::new("/input/subdir/image.png");
        let base_dir = Path::new("/input");
        let output = args.output_path_for_dir_entry(input_file, base_dir);
        assert_eq!(output, PathBuf::from("/output/subdir/image.chroma.png"));
    }
}
