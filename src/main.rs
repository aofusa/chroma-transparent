//! chroma-transparent - クロマキー透過処理CLIツール

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
        eprintln!("入力: {:?}", args.input);
        eprintln!("出力: {:?}", args.output_path());
        eprintln!("設定: {:?}", config);
    }

    // 4. 画像を読み込み
    let image = image::open(&args.input)
        .map_err(|e| ChromaError::ImageLoadError {
            path: args.input.display().to_string(),
            source: e,
        })?
        .to_rgba8();

    if config.verbose {
        eprintln!("画像サイズ: {}x{}", image.width(), image.height());
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

    println!("保存完了: {}", output_path.display());
    Ok(())
}
