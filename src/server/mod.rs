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

    println!("Server started at http://{}", addr);

    // サーバを起動
    warp::serve(routes).run(addr).await;

    Ok(())
}

/// 起動情報を表示
fn print_startup_info(config: &ServerConfig) {
    println!("Starting chroma-transparent server...");
    println!("  Address: http://{}:{}", config.host, config.port);
    println!(
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
            println!("  Video processing: enabled");
        } else {
            println!("  Video processing: disabled (use --enable-video to enable)");
        }
    }
    #[cfg(not(feature = "video"))]
    {
        println!("  Video processing: not available (rebuild with --features video)");
    }

    println!();
    println!(
        "  Web UI: http://{}:{}",
        config.host, config.port
    );
    println!(
        "  API docs: http://{}:{}/api/docs",
        config.host, config.port
    );
    println!(
        "  OpenAPI spec: http://{}:{}/api/openapi.json",
        config.host, config.port
    );
    println!();
}

