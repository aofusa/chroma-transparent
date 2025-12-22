//! warpフィルタ定義

use std::convert::Infallible;
use std::sync::Arc;

use tokio::sync::Mutex;
use warp::Filter;

use super::assets;
use super::config::ServerConfig;
use super::error::handle_rejection;
use super::handlers;
use super::openapi;
use super::storage::StorageManager;

/// すべてのルートを構築
pub fn routes(
    config: Arc<ServerConfig>,
    storage: Arc<Mutex<StorageManager>>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = Infallible> + Clone {
    // APIルート
    let api = api_routes(config.clone(), storage);

    // 静的ファイル（Web UI）
    let static_files = static_routes();

    // CORSの設定
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "DELETE", "OPTIONS"])
        .allow_headers(vec!["content-type"]);

    // すべてのルートを結合
    api.or(static_files)
        .with(cors)
        .recover(handle_rejection)
}

/// APIルートを構築
fn api_routes(
    config: Arc<ServerConfig>,
    storage: Arc<Mutex<StorageManager>>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let api = warp::path("api");

    api.and(
        health(config.clone())
            .or(get_config(config.clone()))
            .or(openapi_spec(config.clone()))
            .or(swagger_ui())
            .or(preview(config.clone()))
            .or(process(config.clone(), storage.clone()))
            .or(download(storage.clone()))
            .or(delete(storage)),
    )
}

/// 静的ファイルルート
fn static_routes() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    // ルート: index.html
    let index = warp::path::end()
        .and(warp::get())
        .map(|| warp::reply::html(assets::INDEX_HTML));

    // style.css
    let css = warp::path("style.css")
        .and(warp::get())
        .map(|| {
            warp::reply::with_header(
                assets::STYLE_CSS,
                "content-type",
                "text/css; charset=utf-8",
            )
        });

    // app.js
    let js = warp::path("app.js")
        .and(warp::get())
        .map(|| {
            warp::reply::with_header(
                assets::APP_JS,
                "content-type",
                "application/javascript; charset=utf-8",
            )
        });

    index.or(css).or(js)
}

// === 個別フィルタ ===

/// ヘルスチェック: GET /api/health
fn health(
    config: Arc<ServerConfig>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("health")
        .and(warp::get())
        .and(with_config(config))
        .map(|config: Arc<ServerConfig>| {
            warp::reply::json(&handlers::health_check(config))
        })
}

/// 設定取得: GET /api/config
fn get_config(
    config: Arc<ServerConfig>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("config")
        .and(warp::get())
        .and(with_config(config))
        .map(|config: Arc<ServerConfig>| {
            warp::reply::json(&handlers::get_config(config))
        })
}

/// OpenAPIスキーマ: GET /api/openapi.json
fn openapi_spec(
    config: Arc<ServerConfig>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("openapi.json")
        .and(warp::get())
        .and(with_config(config))
        .map(|config: Arc<ServerConfig>| {
            let spec = openapi::generate_openapi_spec(config.video_enabled);
            warp::reply::json(&spec)
        })
}

/// Swagger UI: GET /api/docs
fn swagger_ui() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("docs")
        .and(warp::path::end())
        .and(warp::get())
        .map(|| {
            warp::reply::html(openapi::swagger_ui_html())
        })
}

/// プレビュー生成: POST /api/preview
fn preview(
    config: Arc<ServerConfig>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("preview")
        .and(warp::post())
        .and(warp::multipart::form().max_length(50 * 1024 * 1024)) // 50MB
        .and(with_config(config))
        .and_then(handlers::handle_preview)
}

/// 画像処理: POST /api/process
fn process(
    config: Arc<ServerConfig>,
    storage: Arc<Mutex<StorageManager>>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("process")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::multipart::form().max_length(100 * 1024 * 1024)) // 100MB
        .and(with_config(config))
        .and(with_storage(storage))
        .and_then(handlers::handle_process)
}

/// ファイルダウンロード: GET /api/download/{file_id}
fn download(
    storage: Arc<Mutex<StorageManager>>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("download" / String)
        .and(warp::get())
        .and(with_storage(storage))
        .and_then(|file_id: String, storage: Arc<Mutex<StorageManager>>| {
            handlers::handle_download(file_id, storage)
        })
}

/// ファイル削除: DELETE /api/files/{file_id}
fn delete(
    storage: Arc<Mutex<StorageManager>>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("files" / String)
        .and(warp::delete())
        .and(with_storage(storage))
        .and_then(|file_id: String, storage: Arc<Mutex<StorageManager>>| {
            handlers::handle_delete(file_id, storage)
        })
}

// === ヘルパーフィルタ ===

/// 設定を注入
fn with_config(
    config: Arc<ServerConfig>,
) -> impl Filter<Extract = (Arc<ServerConfig>,), Error = Infallible> + Clone {
    warp::any().map(move || config.clone())
}

/// ストレージを注入
fn with_storage(
    storage: Arc<Mutex<StorageManager>>,
) -> impl Filter<Extract = (Arc<Mutex<StorageManager>>,), Error = Infallible> + Clone {
    warp::any().map(move || storage.clone())
}

