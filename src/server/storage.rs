//! ファイルストレージ管理

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

/// ストレージモード
enum StorageMode {
    /// 一時ディレクトリ（サーバ停止時に自動削除）
    Temporary(TempDir),
    /// 永続ディレクトリ（削除しない）
    Persistent(PathBuf),
}

/// ファイル情報
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// ファイルID
    pub id: String,
    /// ファイルパス
    pub path: PathBuf,
    /// ファイルサイズ
    pub size: u64,
    /// 作成日時
    pub created_at: chrono::DateTime<Utc>,
    /// MIMEタイプ
    pub mime_type: String,
}

/// ストレージマネージャー
pub struct StorageManager {
    /// ストレージモード
    mode: StorageMode,
    /// ファイル情報のキャッシュ
    files: HashMap<String, FileInfo>,
}

impl StorageManager {
    /// 新しいストレージマネージャーを作成
    pub fn new(storage_dir: Option<PathBuf>) -> Result<Self> {
        let mode = match storage_dir {
            Some(dir) => {
                // 永続ディレクトリを作成
                fs::create_dir_all(&dir)
                    .with_context(|| format!("Failed to create storage directory: {:?}", dir))?;
                StorageMode::Persistent(dir)
            }
            None => {
                // 一時ディレクトリを作成
                let temp_dir = TempDir::new()
                    .context("Failed to create temporary directory")?;
                StorageMode::Temporary(temp_dir)
            }
        };

        Ok(Self {
            mode,
            files: HashMap::new(),
        })
    }

    /// ストレージディレクトリのパスを取得
    pub fn storage_path(&self) -> &Path {
        match &self.mode {
            StorageMode::Temporary(temp_dir) => temp_dir.path(),
            StorageMode::Persistent(path) => path,
        }
    }

    /// ファイルを保存
    ///
    /// # Arguments
    /// * `data` - ファイルデータ
    /// * `extension` - 拡張子（例: "png", "webm"）
    ///
    /// # Returns
    /// ファイルID
    pub fn save(&mut self, data: &[u8], extension: &str) -> Result<String> {
        // ファイルIDを生成: YYYYMMDD_HHMMSS_hash8
        let now = Utc::now();
        let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
        let hash = compute_hash(data);
        let file_id = format!("{}_{}", timestamp, &hash[..8]);
        let filename = format!("{}.{}", file_id, extension);

        // ファイルパスを構築
        let file_path = self.storage_path().join(&filename);

        // ファイルを書き込み
        let mut file = fs::File::create(&file_path)
            .with_context(|| format!("Failed to create file: {:?}", file_path))?;
        file.write_all(data)
            .with_context(|| format!("Failed to write file: {:?}", file_path))?;

        // MIMEタイプを推定
        let mime_type = match extension {
            "png" => "image/png",
            "webm" => "video/webm",
            "mov" => "video/quicktime",
            _ => "application/octet-stream",
        }
        .to_string();

        // ファイル情報を保存
        let info = FileInfo {
            id: file_id.clone(),
            path: file_path,
            size: data.len() as u64,
            created_at: now,
            mime_type,
        };
        self.files.insert(file_id.clone(), info);

        Ok(file_id)
    }

    /// ファイル情報を取得
    pub fn get(&self, file_id: &str) -> Option<&FileInfo> {
        self.files.get(file_id)
    }

    /// ファイルパスを取得
    pub fn get_path(&self, file_id: &str) -> Option<&Path> {
        self.files.get(file_id).map(|info| info.path.as_path())
    }

    /// ファイルを削除
    pub fn delete(&mut self, file_id: &str) -> Result<bool> {
        if let Some(info) = self.files.remove(file_id) {
            if info.path.exists() {
                fs::remove_file(&info.path)
                    .with_context(|| format!("Failed to delete file: {:?}", info.path))?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// すべてのファイル情報を取得
    #[allow(dead_code)]
    pub fn list(&self) -> Vec<&FileInfo> {
        self.files.values().collect()
    }
}

/// データのSHA-256ハッシュを計算
fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// 16進数エンコード（sha2の結果用）
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_manager_temp() {
        let mut manager = StorageManager::new(None).unwrap();
        
        let data = b"test data";
        let file_id = manager.save(data, "png").unwrap();
        
        assert!(file_id.contains("_"));
        assert!(manager.get(&file_id).is_some());
        
        let info = manager.get(&file_id).unwrap();
        assert_eq!(info.size, 9);
        assert_eq!(info.mime_type, "image/png");
    }

    #[test]
    fn test_compute_hash() {
        let hash = compute_hash(b"hello");
        assert_eq!(hash.len(), 64); // SHA-256 = 32 bytes = 64 hex chars
    }
}

