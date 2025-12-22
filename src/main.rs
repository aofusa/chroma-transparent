//! chroma-transparent - クロマキー透過処理CLIツール

// mimallocをグローバルアロケータとして使用 (macOS, FreeBSD以外)
#[cfg(not(any(target_os = "macos", target_os = "freebsd")))]
use mimalloc::MiMalloc;

#[cfg(not(any(target_os = "macos", target_os = "freebsd")))]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use std::path::Path;

use clap::Parser;

use chroma_transparent::{
    ensure_output_directory, Args, ChromaError, ChromaPipeline, ImageScanner, ProcessConfig,
};

fn main() -> anyhow::Result<()> {
    // 1. CLI引数をパース
    let args = Args::parse();

    // 2. 入力パスの存在確認
    args.validate_input()?;

    // 3. 設定を構築
    let config = ProcessConfig::from_cli(&args)?;
    config.validate()?;

    // 4. 入力がディレクトリかファイルかで処理を分岐
    if args.is_input_directory() {
        process_directory(&args, &config)?;
    } else {
        process_single_file(&args, &config)?;
    }

    Ok(())
}

/// 単一ファイルを処理
fn process_single_file(args: &Args, config: &ProcessConfig) -> anyhow::Result<()> {
    let input_path = &args.input;
    let output_path = args.output_path_for_file(input_path);

    if config.verbose {
        eprintln!("Input: {:?}", input_path);
        eprintln!("Output: {:?}", output_path);
        eprintln!("Config: {:?}", config);
    }

    // 出力ディレクトリを作成
    ensure_output_directory(&output_path)?;

    // 画像を読み込み
    let image = image::open(input_path)
        .map_err(|e| ChromaError::ImageLoadError {
            path: input_path.display().to_string(),
            source: e,
        })?
        .to_rgba8();

    if config.verbose {
        eprintln!("Image size: {}x{}", image.width(), image.height());
    }

    // パイプラインで処理
    let pipeline = ChromaPipeline::new(config.clone());
    let result = pipeline.process(&image);

    // 保存
    result
        .save(&output_path)
        .map_err(|e| ChromaError::ImageSaveError {
            path: output_path.display().to_string(),
            source: e,
        })?;

    println!("Saved: {}", output_path.display());
    Ok(())
}

/// ディレクトリ内のすべての画像を処理
fn process_directory(args: &Args, config: &ProcessConfig) -> anyhow::Result<()> {
    let input_dir = &args.input;

    if config.verbose {
        eprintln!("Input directory: {:?}", input_dir);
        if let Some(output) = &args.output {
            eprintln!("Output directory: {:?}", output);
        }
        eprintln!("Recursive depth: {:?}", args.recursive);
        eprintln!("Config: {:?}", config);
    }

    // 画像ファイルを探索
    let scanner = ImageScanner::new(args.recursive);
    let files = scanner.scan(input_dir);

    if files.is_empty() {
        println!("No image files found in {:?}", input_dir);
        return Ok(());
    }

    println!("Found {} image file(s)", files.len());

    // パイプラインを作成
    let pipeline = ChromaPipeline::new(config.clone());

    // 各ファイルを処理
    let mut success_count = 0;
    let mut error_count = 0;

    for input_path in &files {
        let output_path = args.output_path_for_dir_entry(input_path, input_dir);

        if config.verbose {
            eprintln!("Processing: {:?} -> {:?}", input_path, output_path);
        }

        // 出力ディレクトリを作成
        if let Err(e) = ensure_output_directory(&output_path) {
            eprintln!("Error creating directory for {:?}: {}", output_path, e);
            error_count += 1;
            continue;
        }

        // 画像を処理
        match process_file(&pipeline, input_path, &output_path) {
            Ok(()) => {
                println!("Saved: {}", output_path.display());
                success_count += 1;
            }
            Err(e) => {
                eprintln!("Error processing {:?}: {}", input_path, e);
                error_count += 1;
            }
        }
    }

    // サマリーを出力
    println!();
    println!("Completed: {} succeeded, {} failed", success_count, error_count);

    Ok(())
}

/// 単一ファイルを処理（内部関数）
fn process_file(
    pipeline: &ChromaPipeline,
    input_path: &Path,
    output_path: &Path,
) -> anyhow::Result<()> {
    // 画像を読み込み
    let image = image::open(input_path)
        .map_err(|e| ChromaError::ImageLoadError {
            path: input_path.display().to_string(),
            source: e,
        })?
        .to_rgba8();

    // 処理
    let result = pipeline.process(&image);

    // 保存
    result
        .save(output_path)
        .map_err(|e| ChromaError::ImageSaveError {
            path: output_path.display().to_string(),
            source: e,
        })?;

    Ok(())
}
