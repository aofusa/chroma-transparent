//! リクエストハンドラ

use std::io::Cursor;
use std::sync::Arc;
use std::time::Instant;

use bytes::Buf;
use futures::{StreamExt, TryStreamExt};
use image::{DynamicImage, GenericImageView, ImageFormat};
use log::{debug, info, trace, warn};
use serde::Serialize;
use tokio::sync::Mutex;
use warp::multipart::FormData;
use warp::Rejection;

use crate::color::{ColorSpace, Rgb};
use crate::config::ColorConfig;
use crate::pipeline::ChromaPipeline;
use crate::ProcessConfig;

use super::config::ServerConfig;
use super::error::ApiError;
use super::storage::StorageManager;

/// プレビュー/処理パラメータ
#[derive(Debug, Default)]
pub struct ProcessParams {
    pub image_data: Option<Vec<u8>>,
    pub color: String,
    pub tolerance: f32,
    pub feather: u32,
    pub despill: f32,
    pub erode: u32,
    pub dilate: u32,
    pub preview_size: Option<u32>,
    
    // 新規フィールド
    pub multi_colors: Vec<String>, // "COLOR:TOLERANCE"形式の文字列
    pub color_space: Option<String>,
    
    pub bilateral: Option<bool>,
    pub bilateral_spatial_sigma: Option<f32>,
    pub bilateral_color_sigma: Option<f32>,
    pub bilateral_radius: Option<u32>,
    
    pub multiscale: Option<bool>,
    pub multiscale_levels: Option<u32>,
    pub multiscale_scale_factor: Option<f32>,
    
    pub edge_optimization: Option<bool>,
    pub edge_threshold: Option<f32>,
    pub edge_smoothness: Option<f32>,
    
    pub shadow_removal: Option<bool>,
    pub shadow_threshold: Option<f32>,
    pub shadow_removal_strength: Option<f32>,
    
    pub sharpen: Option<bool>,
    pub sharpen_amount: Option<f32>,
    pub sharpen_radius: Option<f32>,
    pub sharpen_threshold: Option<f32>,
    
    // 新規実装機能
    pub adaptive_tolerance: Option<bool>,
    pub adaptive_tolerance_grid_w: Option<u32>,
    pub adaptive_tolerance_grid_h: Option<u32>,
    pub adaptive_tolerance_sensitivity: Option<f32>,
    
    pub edge_detection_method: Option<String>,
    pub canny_low_threshold: Option<f32>,
    pub canny_high_threshold: Option<f32>,
    pub canny_gaussian_sigma: Option<f32>,
    
    pub despill_method: Option<String>,
    
    pub thin_line_detection: Option<bool>,
    pub thin_line_sensitivity: Option<f32>,
    pub thin_line_threshold: Option<f32>,
    
    pub auto_params: Option<bool>,
}

/// 処理レスポンス
#[derive(Serialize)]
pub struct ProcessResponse {
    pub success: bool,
    pub file_id: String,
    pub download_url: String,
    pub filename: String,
    pub processing_time_ms: u64,
    pub file_size: u64,
}

/// ヘルスチェックレスポンス
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub video_enabled: bool,
}

/// 設定レスポンス
#[derive(Serialize)]
pub struct ConfigResponse {
    pub colors: Vec<String>,
    pub parameters: ParameterRanges,
    pub video_enabled: bool,
}

#[derive(Serialize)]
pub struct ParameterRanges {
    pub tolerance: ParameterRange,
    pub feather: ParameterRange,
    pub despill: ParameterRange,
    pub erode: ParameterRange,
    pub dilate: ParameterRange,
    pub color_space: Vec<String>,
    pub bilateral_spatial_sigma: ParameterRange,
    pub bilateral_color_sigma: ParameterRange,
    pub bilateral_radius: ParameterRange,
    pub multiscale_levels: ParameterRange,
    pub multiscale_scale_factor: ParameterRange,
    pub edge_threshold: ParameterRange,
    pub edge_smoothness: ParameterRange,
    pub shadow_threshold: ParameterRange,
    pub shadow_removal_strength: ParameterRange,
    pub sharpen_amount: ParameterRange,
    pub sharpen_radius: ParameterRange,
    pub sharpen_threshold: ParameterRange,
    pub adaptive_tolerance_grid_w: ParameterRange,
    pub adaptive_tolerance_grid_h: ParameterRange,
    pub adaptive_tolerance_sensitivity: ParameterRange,
}

#[derive(Serialize)]
pub struct ParameterRange {
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub step: f32,
}

/// 削除レスポンス
#[derive(Serialize)]
pub struct DeleteResponse {
    pub success: bool,
    pub file_id: String,
}

/// ヘルスチェック
pub fn health_check(config: Arc<ServerConfig>) -> HealthResponse {
    HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        video_enabled: config.video_enabled,
    }
}

/// 設定を取得
pub fn get_config(config: Arc<ServerConfig>) -> ConfigResponse {
    // よく使う色のリスト
    let colors = vec![
        "lime", "green", "blue", "magenta", "cyan", "red", "yellow", "white", "black",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    ConfigResponse {
        colors,
        parameters: ParameterRanges {
            tolerance: ParameterRange {
                min: 0.0,
                max: 1.0,
                default: 0.3,
                step: 0.01,
            },
            feather: ParameterRange {
                min: 0.0,
                max: 50.0,
                default: 5.0,
                step: 1.0,
            },
            despill: ParameterRange {
                min: 0.0,
                max: 1.0,
                default: 0.7,
                step: 0.01,
            },
            erode: ParameterRange {
                min: 0.0,
                max: 10.0,
                default: 0.0,
                step: 1.0,
            },
            dilate: ParameterRange {
                min: 0.0,
                max: 10.0,
                default: 1.0,
                step: 1.0,
            },
            color_space: vec!["hsv".to_string(), "lab".to_string(), "lch".to_string(), "yuv".to_string()],
            bilateral_spatial_sigma: ParameterRange {
                min: 1.0,
                max: 20.0,
                default: 5.0,
                step: 0.1,
            },
            bilateral_color_sigma: ParameterRange {
                min: 10.0,
                max: 100.0,
                default: 50.0,
                step: 1.0,
            },
            bilateral_radius: ParameterRange {
                min: 1.0,
                max: 10.0,
                default: 5.0,
                step: 1.0,
            },
            multiscale_levels: ParameterRange {
                min: 1.0,
                max: 5.0,
                default: 3.0,
                step: 1.0,
            },
            multiscale_scale_factor: ParameterRange {
                min: 0.25,
                max: 0.75,
                default: 0.5,
                step: 0.05,
            },
            edge_threshold: ParameterRange {
                min: 0.0,
                max: 1.0,
                default: 0.1,
                step: 0.01,
            },
            edge_smoothness: ParameterRange {
                min: 0.0,
                max: 1.0,
                default: 0.5,
                step: 0.01,
            },
            shadow_threshold: ParameterRange {
                min: 0.0,
                max: 1.0,
                default: 0.3,
                step: 0.01,
            },
            shadow_removal_strength: ParameterRange {
                min: 0.0,
                max: 1.0,
                default: 0.7,
                step: 0.01,
            },
            sharpen_amount: ParameterRange {
                min: 0.0,
                max: 2.0,
                default: 0.5,
                step: 0.1,
            },
            sharpen_radius: ParameterRange {
                min: 0.1,
                max: 5.0,
                default: 1.0,
                step: 0.1,
            },
            sharpen_threshold: ParameterRange {
                min: 0.0,
                max: 1.0,
                default: 0.0,
                step: 0.01,
            },
            adaptive_tolerance_grid_w: ParameterRange {
                min: 4.0,
                max: 32.0,
                default: 8.0,
                step: 1.0,
            },
            adaptive_tolerance_grid_h: ParameterRange {
                min: 4.0,
                max: 32.0,
                default: 8.0,
                step: 1.0,
            },
            adaptive_tolerance_sensitivity: ParameterRange {
                min: 0.0,
                max: 2.0,
                default: 1.0,
                step: 0.1,
            },
        },
        video_enabled: config.video_enabled,
    }
}

/// プレビュー生成
pub async fn handle_preview(
    form: FormData,
    config: Arc<ServerConfig>,
) -> Result<impl warp::Reply, Rejection> {
    debug!("Preview handler started");

    // パラメータを解析
    let mut params = parse_multipart_form(form).await?;
    debug!("Multipart form parsed: color={}, tolerance={}", params.color, params.tolerance);

    // 処理設定を構築（image_dataを取り出す前に行う）
    let process_config = build_process_config(&params, config.as_ref())?;
    let preview_size = params.preview_size.unwrap_or(512);
    debug!("Process config built, preview_size={}", preview_size);

    // 画像データを取得
    let image_data = params
        .image_data
        .take()
        .ok_or_else(|| {
            warn!("No image data in form");
            warp::reject::custom(ApiError::BadRequest("No image provided".into()))
        })?;
    debug!("Image data extracted: {} bytes", image_data.len());

    // 画像を読み込み
    let image = load_image(&image_data)?;
    debug!("Image loaded: {}x{}", image.width(), image.height());

    // プレビューサイズにリサイズ
    let image = resize_image(image, preview_size);
    trace!("Image resized to: {}x{}", image.width(), image.height());

    // クロマキー処理
    let pipeline = ChromaPipeline::new(process_config);
    let result = pipeline.process(&image.to_rgba8());
    debug!("Chroma key processing completed");

    // PNGとしてエンコード
    let png_data = encode_to_png(&result)?;
    trace!("PNG encoded: {} bytes", png_data.len());

    // レスポンス
    Ok(warp::reply::with_header(
        png_data,
        "content-type",
        "image/png",
    ))
}

/// フル画像処理
pub async fn handle_process(
    form: FormData,
    config: Arc<ServerConfig>,
    storage: Arc<Mutex<StorageManager>>,
) -> Result<impl warp::Reply, Rejection> {
    debug!("Process handler started");
    let start = Instant::now();

    // パラメータを解析
    let mut params = parse_multipart_form(form).await?;
    debug!("Multipart form parsed: color={}, tolerance={}", params.color, params.tolerance);

    // 処理設定を構築（image_dataを取り出す前に行う）
    let process_config = build_process_config(&params, config.as_ref())?;
    debug!("Process config built");

    // 画像データを取得
    let image_data = params
        .image_data
        .take()
        .ok_or_else(|| {
            warn!("No image data in form");
            warp::reject::custom(ApiError::BadRequest("No image provided".into()))
        })?;
    debug!("Image data extracted: {} bytes", image_data.len());

    // 画像を読み込み
    let image = load_image(&image_data)?;
    debug!("Image loaded: {}x{}", image.width(), image.height());

    // クロマキー処理
    let pipeline = ChromaPipeline::new(process_config);
    let result = pipeline.process(&image.to_rgba8());
    debug!("Chroma key processing completed");

    // PNGとしてエンコード
    let png_data = encode_to_png(&result)?;
    trace!("PNG encoded: {} bytes", png_data.len());

    // ストレージに保存
    let file_id = storage
        .lock()
        .await
        .save(&png_data, "png")
        .map_err(|e| {
            log::error!("Storage save failed: {}", e);
            warp::reject::custom(ApiError::InternalError(e.to_string()))
        })?;
    debug!("File saved with id: {}", file_id);

    let processing_time = start.elapsed().as_millis() as u64;

    // レスポンス
    let response = ProcessResponse {
        success: true,
        file_id: file_id.clone(),
        download_url: format!("/api/download/{}", file_id),
        filename: format!("{}.png", file_id),
        processing_time_ms: processing_time,
        file_size: png_data.len() as u64,
    };

    info!("Process completed in {}ms", processing_time);
    Ok(warp::reply::json(&response))
}

/// ファイルダウンロード
pub async fn handle_download(
    file_id: String,
    storage: Arc<Mutex<StorageManager>>,
) -> Result<impl warp::Reply, Rejection> {
    let storage = storage.lock().await;

    let info = storage
        .get(&file_id)
        .ok_or_else(|| warp::reject::custom(ApiError::NotFound("File not found".into())))?;

    // ファイルを読み込み
    let data = std::fs::read(&info.path)
        .map_err(|e| warp::reject::custom(ApiError::InternalError(e.to_string())))?;

    let mime_type = info.mime_type.clone();
    let filename = info.path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download")
        .to_string();

    // Content-Disposition ヘッダーを追加
    Ok(warp::reply::with_header(
        warp::reply::with_header(data, "content-type", mime_type),
        "content-disposition",
        format!("attachment; filename=\"{}\"", filename),
    ))
}

/// ファイル削除
pub async fn handle_delete(
    file_id: String,
    storage: Arc<Mutex<StorageManager>>,
) -> Result<impl warp::Reply, Rejection> {
    let deleted = storage
        .lock()
        .await
        .delete(&file_id)
        .map_err(|e| warp::reject::custom(ApiError::InternalError(e.to_string())))?;

    if deleted {
        Ok(warp::reply::json(&DeleteResponse {
            success: true,
            file_id,
        }))
    } else {
        Err(warp::reject::custom(ApiError::NotFound(
            "File not found".into(),
        )))
    }
}

// === ヘルパー関数 ===

/// マルチパートフォームを解析
async fn parse_multipart_form(mut form: FormData) -> Result<ProcessParams, Rejection> {
    trace!("parse_multipart_form started");
    
    let mut params = ProcessParams {
        color: "lime".to_string(),
        tolerance: 0.3,
        feather: 5,
        despill: 0.7,
        erode: 0,
        dilate: 1,
        multi_colors: Vec::new(),
        color_space: None,
        bilateral: None,
        bilateral_spatial_sigma: None,
        bilateral_color_sigma: None,
        bilateral_radius: None,
        multiscale: None,
        multiscale_levels: None,
        multiscale_scale_factor: None,
        edge_optimization: None,
        edge_threshold: None,
        edge_smoothness: None,
        shadow_removal: None,
        shadow_threshold: None,
        shadow_removal_strength: None,
        sharpen: None,
        sharpen_amount: None,
        sharpen_radius: None,
        sharpen_threshold: None,
        
        adaptive_tolerance: None,
        adaptive_tolerance_grid_w: None,
        adaptive_tolerance_grid_h: None,
        adaptive_tolerance_sensitivity: None,
        
        edge_detection_method: None,
        canny_low_threshold: None,
        canny_high_threshold: None,
        canny_gaussian_sigma: None,
        
        despill_method: None,
        
        thin_line_detection: None,
        thin_line_sensitivity: None,
        thin_line_threshold: None,
        
        auto_params: None,
        ..Default::default()
    };

    // パートを一つずつ処理（try_collect()ではなくnext()を使用）
    // warpのmultipartではtry_collect()を使うと "failed to lock multipart state" エラーが発生することがある
    let mut part_count = 0;
    while let Some(part_result) = form.next().await {
        let part = part_result.map_err(|e| {
            warn!("Failed to get form part: {}", e);
            warp::reject::custom(ApiError::BadRequest(e.to_string()))
        })?;
        
        let name = part.name().to_string();
        trace!("Processing form part: {}", name);
        
        // パートのデータを読み込み
        let data = part
            .stream()
            .try_fold(Vec::new(), |mut acc, buf| async move {
                acc.extend_from_slice(buf.chunk());
                Ok(acc)
            })
            .await
            .map_err(|e| {
                warn!("Failed to read part '{}': {}", name, e);
                warp::reject::custom(ApiError::BadRequest(e.to_string()))
            })?;
        
        trace!("Part '{}' data size: {} bytes", name, data.len());
        part_count += 1;

        match name.as_str() {
            "image" | "file" => {
                params.image_data = Some(data);
            }
            "color" => {
                if let Ok(s) = String::from_utf8(data) {
                    params.color = s.trim().to_string();
                }
            }
            "tolerance" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.tolerance = v;
                    }
                }
            }
            "feather" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.feather = v;
                    }
                }
            }
            "despill" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.despill = v;
                    }
                }
            }
            "erode" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.erode = v;
                    }
                }
            }
            "dilate" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.dilate = v;
                    }
                }
            }
            "preview_size" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.preview_size = Some(v);
                    }
                }
            }
            "color_space" => {
                if let Ok(s) = String::from_utf8(data) {
                    params.color_space = Some(s.trim().to_string());
                }
            }
            "multi_color" => {
                if let Ok(s) = String::from_utf8(data) {
                    params.multi_colors.push(s.trim().to_string());
                }
            }
            "bilateral" => {
                params.bilateral = Some(true);
            }
            "bilateral_spatial_sigma" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.bilateral_spatial_sigma = Some(v);
                    }
                }
            }
            "bilateral_color_sigma" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.bilateral_color_sigma = Some(v);
                    }
                }
            }
            "bilateral_radius" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.bilateral_radius = Some(v);
                    }
                }
            }
            "multiscale" => {
                params.multiscale = Some(true);
            }
            "multiscale_levels" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.multiscale_levels = Some(v);
                    }
                }
            }
            "multiscale_scale_factor" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.multiscale_scale_factor = Some(v);
                    }
                }
            }
            "edge_optimization" => {
                params.edge_optimization = Some(true);
            }
            "edge_threshold" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.edge_threshold = Some(v);
                    }
                }
            }
            "edge_smoothness" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.edge_smoothness = Some(v);
                    }
                }
            }
            "shadow_removal" => {
                params.shadow_removal = Some(true);
            }
            "shadow_threshold" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.shadow_threshold = Some(v);
                    }
                }
            }
            "shadow_removal_strength" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.shadow_removal_strength = Some(v);
                    }
                }
            }
            "sharpen" => {
                params.sharpen = Some(true);
            }
            "sharpen_amount" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.sharpen_amount = Some(v);
                    }
                }
            }
            "sharpen_radius" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.sharpen_radius = Some(v);
                    }
                }
            }
            "sharpen_threshold" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.sharpen_threshold = Some(v);
                    }
                }
            }
            "adaptive_tolerance" => {
                params.adaptive_tolerance = Some(true);
            }
            "adaptive_tolerance_grid_w" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.adaptive_tolerance_grid_w = Some(v);
                    }
                }
            }
            "adaptive_tolerance_grid_h" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.adaptive_tolerance_grid_h = Some(v);
                    }
                }
            }
            "adaptive_tolerance_sensitivity" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.adaptive_tolerance_sensitivity = Some(v);
                    }
                }
            }
            "edge_detection_method" => {
                if let Ok(s) = String::from_utf8(data) {
                    params.edge_detection_method = Some(s.trim().to_string());
                }
            }
            "canny_low_threshold" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.canny_low_threshold = Some(v);
                    }
                }
            }
            "canny_high_threshold" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.canny_high_threshold = Some(v);
                    }
                }
            }
            "canny_gaussian_sigma" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.canny_gaussian_sigma = Some(v);
                    }
                }
            }
            "despill_method" => {
                if let Ok(s) = String::from_utf8(data) {
                    params.despill_method = Some(s.trim().to_string());
                }
            }
            "thin_line_detection" => {
                params.thin_line_detection = Some(true);
            }
            "thin_line_sensitivity" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.thin_line_sensitivity = Some(v);
                    }
                }
            }
            "thin_line_threshold" => {
                if let Ok(s) = String::from_utf8(data) {
                    if let Ok(v) = s.trim().parse() {
                        params.thin_line_threshold = Some(v);
                    }
                }
            }
            "auto_params" => {
                params.auto_params = Some(true);
            }
            _ => {}
        }
    }
    
    trace!("Parsed {} form parts", part_count);

    Ok(params)
}

/// 処理設定を構築
fn build_process_config(params: &ProcessParams, _config: &ServerConfig) -> Result<ProcessConfig, Rejection> {
    let chroma_color = Rgb::from_color_spec(&params.color)
        .map_err(|e| warp::reject::custom(ApiError::BadRequest(e.to_string())))?;

    // 多色検出の設定
    let multi_colors = if params.multi_colors.is_empty() {
        None
    } else {
        Some(
            params.multi_colors
                .iter()
                .map(|s| {
                    let parts: Vec<&str> = s.split(':').collect();
                    if parts.len() != 2 {
                        return Err(warp::reject::custom(ApiError::BadRequest(
                            format!("Invalid multi-color format: {}", s)
                        )));
                    }
                    let color = Rgb::from_color_spec(parts[0])
                        .map_err(|e| warp::reject::custom(ApiError::BadRequest(e.to_string())))?;
                    let tolerance = parts[1]
                        .parse::<f32>()
                        .map_err(|_| warp::reject::custom(ApiError::BadRequest(
                            format!("Invalid tolerance: {}", parts[1])
                        )))?;
                    Ok(ColorConfig { color, tolerance })
                })
                .collect::<Result<Vec<_>, _>>()?,
        )
    };

    // 色空間の設定
    let color_space = if let Some(ref cs) = params.color_space {
        ColorSpace::from_str(cs)
            .map_err(|e| warp::reject::custom(ApiError::BadRequest(e)))?
    } else {
        ColorSpace::Hsv
    };

    Ok(ProcessConfig {
        chroma_color,
        tolerance: params.tolerance,
        feather_amount: params.feather,
        despill_strength: params.despill,
        erode_iterations: params.erode,
        dilate_iterations: params.dilate,
        verbose: false,
        
        multi_colors,
        color_space,
        
        bilateral_enabled: params.bilateral.unwrap_or(false),
        bilateral_spatial_sigma: params.bilateral_spatial_sigma.unwrap_or(5.0),
        bilateral_color_sigma: params.bilateral_color_sigma.unwrap_or(50.0),
        bilateral_radius: params.bilateral_radius.unwrap_or(5),
        
        multiscale_enabled: params.multiscale.unwrap_or(false),
        multiscale_levels: params.multiscale_levels.unwrap_or(3),
        multiscale_scale_factor: params.multiscale_scale_factor.unwrap_or(0.5),
        
        edge_optimization_enabled: params.edge_optimization.unwrap_or(false),
        edge_threshold: params.edge_threshold.unwrap_or(0.1),
        edge_smoothness: params.edge_smoothness.unwrap_or(0.5),
        
        shadow_removal_enabled: params.shadow_removal.unwrap_or(false),
        shadow_threshold: params.shadow_threshold.unwrap_or(0.3),
        shadow_removal_strength: params.shadow_removal_strength.unwrap_or(0.7),
        
        sharpen_enabled: params.sharpen.unwrap_or(false),
        sharpen_amount: params.sharpen_amount.unwrap_or(0.5),
        sharpen_radius: params.sharpen_radius.unwrap_or(1.0),
        sharpen_threshold: params.sharpen_threshold.unwrap_or(0.0),
        
        adaptive_tolerance_enabled: params.adaptive_tolerance.unwrap_or(false),
        adaptive_tolerance_grid_w: params.adaptive_tolerance_grid_w.unwrap_or(8),
        adaptive_tolerance_grid_h: params.adaptive_tolerance_grid_h.unwrap_or(8),
        adaptive_tolerance_sensitivity: params.adaptive_tolerance_sensitivity.unwrap_or(1.0),
        
        edge_detection_method: match params.edge_detection_method.as_deref() {
            Some("canny") => crate::processor::EdgeDetectionMethod::Canny,
            _ => crate::processor::EdgeDetectionMethod::Sobel,
        },
        canny_low_threshold: params.canny_low_threshold.unwrap_or(0.1),
        canny_high_threshold: params.canny_high_threshold.unwrap_or(0.3),
        canny_gaussian_sigma: params.canny_gaussian_sigma.unwrap_or(1.0),
        
        despill_method: match params.despill_method.as_deref() {
            Some("advanced") => crate::processor::DespillMethod::Advanced,
            _ => crate::processor::DespillMethod::Basic,
        },
        
        thin_line_detection_enabled: params.thin_line_detection.unwrap_or(false),
        thin_line_sensitivity: params.thin_line_sensitivity.unwrap_or(0.5),
        thin_line_threshold: params.thin_line_threshold.unwrap_or(0.3),
        
        auto_params_enabled: params.auto_params.unwrap_or(false),
    })
}

/// 画像を読み込み
fn load_image(data: &[u8]) -> Result<DynamicImage, Rejection> {
    trace!("Loading image from {} bytes", data.len());
    image::load_from_memory(data)
        .map_err(|e| {
            warn!("Failed to load image: {}", e);
            warp::reject::custom(ApiError::BadRequest(format!("Invalid image: {}", e)))
        })
}

/// 画像をリサイズ
fn resize_image(image: DynamicImage, max_size: u32) -> DynamicImage {
    let (w, h) = image.dimensions();
    if w <= max_size && h <= max_size {
        return image;
    }

    let ratio = (max_size as f32) / (w.max(h) as f32);
    let new_w = (w as f32 * ratio) as u32;
    let new_h = (h as f32 * ratio) as u32;

    image.resize(new_w, new_h, image::imageops::FilterType::Triangle)
}

/// 画像をPNGとしてエンコード
fn encode_to_png(image: &image::RgbaImage) -> Result<Vec<u8>, Rejection> {
    trace!("Encoding image to PNG: {}x{}", image.width(), image.height());
    let mut buf = Cursor::new(Vec::new());
    image
        .write_to(&mut buf, ImageFormat::Png)
        .map_err(|e| {
            log::error!("Failed to encode PNG: {}", e);
            warp::reject::custom(ApiError::ProcessingError(e.to_string()))
        })?;
    let result = buf.into_inner();
    trace!("PNG encoding completed: {} bytes", result.len());
    Ok(result)
}

