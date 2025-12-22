//! warpフィルタ定義

use std::convert::Infallible;
use std::sync::Arc;

use chrono::Local;
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

    // アクセスログ
    let log = warp::log::custom(|info| {
        let now = Local::now();
        let status = info.status();
        let method = info.method();
        let path = info.path();
        let elapsed = info.elapsed();
        let remote_addr = info
            .remote_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "-".to_string());
        let user_agent = info
            .request_headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("-");

        // ステータスコードに応じて色分け（ターミナル出力用）
        let status_code = status.as_u16();
        let status_str = if status_code >= 500 {
            format!("\x1b[31m{}\x1b[0m", status_code) // Red
        } else if status_code >= 400 {
            format!("\x1b[33m{}\x1b[0m", status_code) // Yellow
        } else if status_code >= 300 {
            format!("\x1b[36m{}\x1b[0m", status_code) // Cyan
        } else {
            format!("\x1b[32m{}\x1b[0m", status_code) // Green
        };

        println!(
            "{} {} {} \"{}\" {} {:.3}ms \"{}\"",
            now.format("%Y-%m-%d %H:%M:%S"),
            remote_addr,
            method,
            path,
            status_str,
            elapsed.as_secs_f64() * 1000.0,
            user_agent
        );
    });

    // すべてのルートを結合
    // 順序: cors → recover → log
    // recoverでrejectionをレスポンスに変換した後、logでログを記録する
    api.or(static_files)
        .with(cors)
        .recover(handle_rejection)
        .with(log)
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

