//! コマンドライン引数パーサー

use clap::Parser;
use std::path::{Path, PathBuf};

use crate::error::{ChromaError, Result};

/// 指定した色をクロマキー処理して透過PNGに変換するCLIツール
#[derive(Parser, Debug)]
#[command(name = "chroma-transparent")]
#[command(version)]
#[command(about = "指定した色をクロマキー処理して透過PNGに変換")]
#[command(long_about = r#"
指定した画像の指定された色をクロマキー処理して透過PNGに変換するCLIツールです。

グリーンバック画像やブルーバック画像など、単色背景の画像から
被写体を切り抜いて透過PNGを生成できます。

ファイルまたはディレクトリを入力として指定できます。
ディレクトリを指定した場合、そのディレクトリ内のすべての画像ファイルを処理します。

色の指定にはHEXコード（00FF00）またはCSS色名（lime, green, blue等）が使用できます。

例:
  chroma-transparent photo.png
  chroma-transparent photo.png -o result.png -c lime
  chroma-transparent photo.png -c blue -t 0.4 -f 10 -d 0.9
  chroma-transparent ./input_dir -o ./output_dir
  chroma-transparent ./input_dir -o ./output_dir -r 2
"#)]
pub struct Args {
    /// 入力画像またはディレクトリのパス
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,

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

    /// 詳細ログを出力
    #[arg(short, long)]
    pub verbose: bool,
}

impl Args {
    /// 入力がディレクトリかどうか
    pub fn is_input_directory(&self) -> bool {
        self.input.is_dir()
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
        self.output_path_for_file(&self.input)
    }

    /// 入力パスの存在確認
    pub fn validate_input(&self) -> Result<()> {
        if !self.input.exists() {
            return Err(ChromaError::InputFileNotFound {
                path: self.input.display().to_string(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_path_default() {
        let args = Args {
            input: PathBuf::from("/path/to/image.png"),
            output: None,
            color: "00FF00".to_string(),
            tolerance: 0.3,
            feather: 5,
            despill: 0.7,
            erode: 0,
            dilate: 1,
            recursive: None,
            verbose: false,
        };
        assert_eq!(
            args.output_path(),
            PathBuf::from("/path/to/image.chroma.png")
        );
    }

    #[test]
    fn test_output_path_specified() {
        let args = Args {
            input: PathBuf::from("/path/to/image.png"),
            output: Some(PathBuf::from("/other/output.png")),
            color: "00FF00".to_string(),
            tolerance: 0.3,
            feather: 5,
            despill: 0.7,
            erode: 0,
            dilate: 1,
            recursive: None,
            verbose: false,
        };
        assert_eq!(args.output_path(), PathBuf::from("/other/output.png"));
    }

    #[test]
    fn test_output_path_for_dir_entry() {
        let args = Args {
            input: PathBuf::from("/input"),
            output: Some(PathBuf::from("/output")),
            color: "00FF00".to_string(),
            tolerance: 0.3,
            feather: 5,
            despill: 0.7,
            erode: 0,
            dilate: 1,
            recursive: Some(0),
            verbose: false,
        };

        // サブディレクトリ内のファイル
        let input_file = Path::new("/input/subdir/image.png");
        let base_dir = Path::new("/input");
        let output = args.output_path_for_dir_entry(input_file, base_dir);
        assert_eq!(output, PathBuf::from("/output/subdir/image.chroma.png"));
    }
}
