//! chroma-transparent - クロマキー透過処理CLIツール

// mimallocをグローバルアロケータとして使用 (macOS, FreeBSD以外)
#[cfg(not(any(target_os = "macos", target_os = "freebsd")))]
use mimalloc::MiMalloc;

#[cfg(not(any(target_os = "macos", target_os = "freebsd")))]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use clap::Parser;

use chroma_transparent::{Args, ChromaError, ChromaPipeline, ProcessConfig};

fn main() -> anyhow::Result<()> {
    // 1. CLI引数をパース
    let args = Args::parse();

    // 2. 入力ファイルの存在確認
    args.validate_input()?;

    // 3. 設定を構築
    let config = ProcessConfig::from_cli(&args)?;
    config.validate()?;

    if config.verbose {
        eprintln!("Input: {:?}", args.input);
        eprintln!("Output: {:?}", args.output_path());
        eprintln!("Config: {:?}", config);
    }

    // 4. 画像を読み込み
    let image = image::open(&args.input)
        .map_err(|e| ChromaError::ImageLoadError {
            path: args.input.display().to_string(),
            source: e,
        })?
        .to_rgba8();

    if config.verbose {
        eprintln!("Image size: {}x{}", image.width(), image.height());
    }

    // 5. パイプラインで処理
    let pipeline = ChromaPipeline::new(config);
    let result = pipeline.process(&image);

    // 6. 保存
    let output_path = args.output_path();
    result
        .save(&output_path)
        .map_err(|e| ChromaError::ImageSaveError {
            path: output_path.display().to_string(),
            source: e,
        })?;

    println!("Saved: {}", output_path.display());
    Ok(())
}
