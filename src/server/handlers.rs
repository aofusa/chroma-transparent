//! リクエストハンドラ

use std::io::Cursor;
use std::sync::Arc;
use std::time::Instant;

use bytes::Buf;
use futures::TryStreamExt;
use image::{DynamicImage, GenericImageView, ImageFormat};
use serde::Serialize;
use tokio::sync::Mutex;
use warp::multipart::{FormData, Part};
use warp::Rejection;

use crate::color::Rgb;
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
        },
        video_enabled: config.video_enabled,
    }
}

/// プレビュー生成
pub async fn handle_preview(
    form: FormData,
    config: Arc<ServerConfig>,
) -> Result<impl warp::Reply, Rejection> {
    // パラメータを解析
    let mut params = parse_multipart_form(form).await?;

    // 処理設定を構築（image_dataを取り出す前に行う）
    let process_config = build_process_config(&params, config.as_ref())?;
    let preview_size = params.preview_size.unwrap_or(512);

    // 画像データを取得
    let image_data = params
        .image_data
        .take()
        .ok_or_else(|| warp::reject::custom(ApiError::BadRequest("No image provided".into())))?;

    // 画像を読み込み
    let image = load_image(&image_data)?;

    // プレビューサイズにリサイズ
    let image = resize_image(image, preview_size);

    // クロマキー処理
    let pipeline = ChromaPipeline::new(process_config);
    let result = pipeline.process(&image.to_rgba8());

    // PNGとしてエンコード
    let png_data = encode_to_png(&result)?;

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
    let start = Instant::now();

    // パラメータを解析
    let mut params = parse_multipart_form(form).await?;

    // 処理設定を構築（image_dataを取り出す前に行う）
    let process_config = build_process_config(&params, config.as_ref())?;

    // 画像データを取得
    let image_data = params
        .image_data
        .take()
        .ok_or_else(|| warp::reject::custom(ApiError::BadRequest("No image provided".into())))?;

    // 画像を読み込み
    let image = load_image(&image_data)?;

    // クロマキー処理
    let pipeline = ChromaPipeline::new(process_config);
    let result = pipeline.process(&image.to_rgba8());

    // PNGとしてエンコード
    let png_data = encode_to_png(&result)?;

    // ストレージに保存
    let file_id = storage
        .lock()
        .await
        .save(&png_data, "png")
        .map_err(|e| warp::reject::custom(ApiError::InternalError(e.to_string())))?;

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
async fn parse_multipart_form(form: FormData) -> Result<ProcessParams, Rejection> {
    let mut params = ProcessParams {
        color: "lime".to_string(),
        tolerance: 0.3,
        feather: 5,
        despill: 0.7,
        erode: 0,
        dilate: 1,
        ..Default::default()
    };

    let parts: Vec<Part> = form
        .try_collect()
        .await
        .map_err(|e| warp::reject::custom(ApiError::BadRequest(e.to_string())))?;

    for part in parts {
        let name = part.name().to_string();
        let data = part
            .stream()
            .try_fold(Vec::new(), |mut acc, buf| async move {
                acc.extend_from_slice(buf.chunk());
                Ok(acc)
            })
            .await
            .map_err(|e| warp::reject::custom(ApiError::BadRequest(e.to_string())))?;

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
            _ => {}
        }
    }

    Ok(params)
}

/// 処理設定を構築
fn build_process_config(params: &ProcessParams, _config: &ServerConfig) -> Result<ProcessConfig, Rejection> {
    let chroma_color = Rgb::from_color_spec(&params.color)
        .map_err(|e| warp::reject::custom(ApiError::BadRequest(e.to_string())))?;

    Ok(ProcessConfig {
        chroma_color,
        tolerance: params.tolerance,
        feather_amount: params.feather,
        despill_strength: params.despill,
        erode_iterations: params.erode,
        dilate_iterations: params.dilate,
        verbose: false,
    })
}

/// 画像を読み込み
fn load_image(data: &[u8]) -> Result<DynamicImage, Rejection> {
    image::load_from_memory(data)
        .map_err(|e| warp::reject::custom(ApiError::BadRequest(format!("Invalid image: {}", e))))
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
    let mut buf = Cursor::new(Vec::new());
    image
        .write_to(&mut buf, ImageFormat::Png)
        .map_err(|e| warp::reject::custom(ApiError::ProcessingError(e.to_string())))?;
    Ok(buf.into_inner())
}

