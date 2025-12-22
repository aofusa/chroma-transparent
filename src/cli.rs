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

例:
  chroma-transparent photo.png
  chroma-transparent photo.png -o result.png -c 00FF00
  chroma-transparent photo.png -t 0.4 -f 10 -d 0.9
"#)]
pub struct Args {
    /// 入力画像のパス
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,

    /// 出力ファイルのパス [デフォルト: <入力ファイル名>.chroma.png]
    #[arg(short, long, value_name = "OUTPUT")]
    pub output: Option<PathBuf>,

    /// クロマキー処理する色 (HEXコード)
    #[arg(short, long, default_value = "00FF00", value_name = "HEX")]
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

    /// 詳細ログを出力
    #[arg(short, long)]
    pub verbose: bool,
}

impl Args {
    /// 出力パスを取得（指定がなければデフォルト生成）
    pub fn output_path(&self) -> PathBuf {
        self.output.clone().unwrap_or_else(|| {
            let stem = self
                .input
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            let parent = self.input.parent().unwrap_or(Path::new("."));
            parent.join(format!("{}.chroma.png", stem))
        })
    }

    /// 入力ファイルの存在確認
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
            verbose: false,
        };
        assert_eq!(args.output_path(), PathBuf::from("/other/output.png"));
    }
}

