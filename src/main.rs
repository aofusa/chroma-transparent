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

#[cfg(feature = "video")]
use chroma_transparent::{
    detect_file_type, generate_video_output_path, FileType, Ffmpeg, VideoFormat,
    VideoProcessConfig, VideoProcessor,
};

#[cfg(feature = "server")]
fn main() -> anyhow::Result<()> {
    // CLI引数をパース
    let args = Args::parse();

    // サーバモードの場合
    if args.is_server_mode() {
        let config = args.build_server_config();
        
        // tokioランタイムを起動
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(chroma_transparent::server::run(config))?;
        
        return Ok(());
    }

    // 通常のCLIモード
    run_cli(args)
}

#[cfg(not(feature = "server"))]
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    run_cli(args)
}

/// CLIモードで実行
fn run_cli(args: Args) -> anyhow::Result<()> {
    // 入力パスの存在確認
    args.validate_input()?;

    // 入力パスを取得
    let input = args.input_path()?;

    // 設定を構築
    let config = ProcessConfig::from_cli(&args)?;
    config.validate()?;

    // 入力の種類を判定して処理
    #[cfg(feature = "video")]
    {
        let file_type = detect_file_type(input);

        match file_type {
            FileType::Directory => process_directory(&args, &config)?,
            FileType::Image => process_single_image(&args, &config)?,
            FileType::Video => process_video(&args, &config)?,
            FileType::Unknown => {
                anyhow::bail!("Unknown file type: {:?}", input);
            }
        }
    }

    #[cfg(not(feature = "video"))]
    {
        if input.is_dir() {
            process_directory(&args, &config)?;
        } else {
            process_single_image(&args, &config)?;
        }
    }

    Ok(())
}

/// 単一画像ファイルを処理
fn process_single_image(args: &Args, config: &ProcessConfig) -> anyhow::Result<()> {
    let input_path = args.input_path()?;
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

/// 動画ファイルを処理
#[cfg(feature = "video")]
fn process_video(args: &Args, config: &ProcessConfig) -> anyhow::Result<()> {
    let input_path = args.input_path()?;

    // 出力フォーマットを決定
    let video_format = if let Some(format_str) = &args.video_format {
        VideoFormat::from_str(format_str)
            .ok_or_else(|| anyhow::anyhow!("Unknown video format: {}", format_str))?
    } else {
        // 出力ファイル名から拡張子を推測
        if let Some(output) = &args.output {
            match output.extension().and_then(|e| e.to_str()) {
                Some("mov") => VideoFormat::Mov,
                Some("webm") => VideoFormat::WebM,
                _ => VideoFormat::WebM,
            }
        } else {
            VideoFormat::WebM
        }
    };

    // 出力パスを決定
    let output_path = if let Some(output) = &args.output {
        output.clone()
    } else {
        generate_video_output_path(input_path, video_format)
    };

    if config.verbose {
        eprintln!("Input video: {:?}", input_path);
        eprintln!("Output: {:?}", output_path);
        eprintln!("Format: {}", video_format);
        eprintln!("Quality: {}", args.video_quality);
        if let Some(fps) = args.fps {
            eprintln!("FPS: {}", fps);
        }
        eprintln!("Config: {:?}", config);
    }

    // 出力ディレクトリを作成
    ensure_output_directory(&output_path)?;

    // ffmpegラッパーを作成
    let ffmpeg = Ffmpeg::new(&args.ffmpeg).with_verbose(config.verbose);

    // ffmpegが利用可能か確認
    if !ffmpeg.is_available() {
        anyhow::bail!(
            "ffmpeg not found at '{}'. Please install ffmpeg or specify the correct path with --ffmpeg",
            args.ffmpeg.display()
        );
    }

    if config.verbose {
        if let Ok(version) = ffmpeg.version() {
            eprintln!("Using: {}", version);
        }
    }

    // クロマキーパイプラインを作成
    let chroma_pipeline = ChromaPipeline::new(config.clone());

    // 動画処理設定
    let video_config = VideoProcessConfig {
        format: video_format,
        quality: args.video_quality,
        fps: args.fps,
        verbose: config.verbose,
    };

    // 動画処理
    let processor = VideoProcessor::new(ffmpeg, chroma_pipeline, video_config);
    processor.process(input_path, &output_path)?;

    println!("Saved: {}", output_path.display());
    Ok(())
}

/// ディレクトリ内のすべての画像を処理
fn process_directory(args: &Args, config: &ProcessConfig) -> anyhow::Result<()> {
    let input_dir = args.input_path()?;

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
