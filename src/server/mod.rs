//! Webサーバモジュール
//!
//! HTTPサーバを起動し、REST APIとWeb UIを提供します。

mod assets;
mod config;
mod error;
mod filters;
mod handlers;
mod openapi;
mod storage;

pub use config::ServerConfig;
pub use error::ApiError;
pub use storage::StorageManager;

use std::net::SocketAddr;
use std::sync::Arc;

use log::{debug, info, warn};
use tokio::sync::Mutex;

/// サーバを起動
pub async fn run(config: ServerConfig) -> anyhow::Result<()> {
    // 起動情報を表示
    print_startup_info(&config);

    // ストレージマネージャーを初期化
    let storage = Arc::new(Mutex::new(StorageManager::new(config.storage_dir.clone())?));

    // 設定を共有
    let config = Arc::new(config);

    // ルートを構築
    let routes = filters::routes(config.clone(), storage);

    // アドレスを解析
    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

    // Graceful Shutdown用のシグナル受信
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    // シグナルハンドラを起動
    tokio::spawn(async move {
        if let Err(e) = shutdown_signal().await {
            warn!("Error waiting for shutdown signal: {}", e);
        }
        let _ = tx.send(());
    });

    info!("Server started at http://{}", addr);
    info!("Press Ctrl+C to stop");

    // Graceful Shutdownでサーバを起動
    let (_, server) = warp::serve(routes)
        .bind_with_graceful_shutdown(addr, async {
            rx.await.ok();
        });

    server.await;

    info!("Server stopped gracefully");
    Ok(())
}

/// シャットダウンシグナルを待機
async fn shutdown_signal() -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigint = signal(SignalKind::interrupt())?;
        let mut sigterm = signal(SignalKind::terminate())?;

        tokio::select! {
            _ = sigint.recv() => {
                info!("Received SIGINT, initiating graceful shutdown...");
            }
            _ = sigterm.recv() => {
                info!("Received SIGTERM, initiating graceful shutdown...");
            }
        }
    }

    #[cfg(windows)]
    {
        tokio::signal::ctrl_c().await?;
        info!("Received Ctrl+C, initiating graceful shutdown...");
    }

    Ok(())
}

/// 起動情報を表示
fn print_startup_info(config: &ServerConfig) {
    info!("Starting chroma-transparent server...");
    info!("  Address: http://{}:{}", config.host, config.port);
    info!(
        "  Storage: {}",
        match &config.storage_dir {
            Some(dir) => format!("{} (persistent)", dir.display()),
            None => "temporary directory (auto-cleanup)".to_string(),
        }
    );

    // video機能の状態表示
    #[cfg(feature = "video")]
    {
        if config.video_enabled {
            info!("  Video processing: enabled");
        } else {
            debug!("  Video processing: disabled (use --enable-video to enable)");
        }
    }
    #[cfg(not(feature = "video"))]
    {
        debug!("  Video processing: not available (rebuild with --features video)");
    }

    info!(
        "  Web UI: http://{}:{}",
        config.host, config.port
    );
    info!(
        "  API docs: http://{}:{}/api/docs",
        config.host, config.port
    );
    debug!(
        "  OpenAPI spec: http://{}:{}/api/openapi.json",
        config.host, config.port
    );
}

