//! サーバ設定

use std::path::PathBuf;

/// サーバ設定
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// リッスンホスト
    pub host: String,
    /// リッスンポート
    pub port: u16,
    /// 処理結果の保存先ディレクトリ（Noneの場合は一時ディレクトリ）
    pub storage_dir: Option<PathBuf>,
    /// 動画処理が有効かどうか（ビルド時 + 実行時の両方の条件が必要）
    pub video_enabled: bool,
    /// CORS許可オリジン
    pub cors_origin: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            storage_dir: None,
            video_enabled: false,
            cors_origin: None,
        }
    }
}

