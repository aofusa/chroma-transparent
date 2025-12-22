//! ディレクトリ内の画像ファイル探索モジュール

use std::fs;
use std::path::{Path, PathBuf};

/// サポートされている画像拡張子
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "bmp", "webp", "tiff", "tif",
];

/// ディレクトリ内の画像ファイルを探索するスキャナー
pub struct ImageScanner {
    /// 再帰的探索の最大深さ (None = 再帰なし, Some(0) = 無制限)
    max_depth: Option<u32>,
}

impl ImageScanner {
    /// 新しいスキャナーを作成
    ///
    /// # Arguments
    /// * `recursive_depth` - 再帰的探索の深さ
    ///   - None: 再帰的探索なし（直下のファイルのみ）
    ///   - Some(0): 無制限に再帰
    ///   - Some(n): 最大n階層まで再帰
    pub fn new(recursive_depth: Option<u32>) -> Self {
        Self {
            max_depth: recursive_depth,
        }
    }

    /// ディレクトリ内の画像ファイルを探索
    pub fn scan(&self, dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        self.scan_recursive(dir, 0, &mut files);
        files.sort();
        files
    }

    /// 再帰的にディレクトリを探索
    fn scan_recursive(&self, dir: &Path, current_depth: u32, files: &mut Vec<PathBuf>) {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() {
                if self.is_supported_image(&path) {
                    files.push(path);
                }
            } else if path.is_dir() {
                // 再帰的探索の判定
                let should_recurse = match self.max_depth {
                    None => false, // 再帰なし
                    Some(0) => true, // 無制限
                    Some(max) => current_depth < max, // 深さ制限あり
                };

                if should_recurse {
                    self.scan_recursive(&path, current_depth + 1, files);
                }
            }
        }
    }

    /// サポートされている画像ファイルかどうか判定
    fn is_supported_image(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
    }
}

/// 出力ディレクトリを作成（親ディレクトリも含めて）
pub fn ensure_output_directory(output_path: &Path) -> std::io::Result<()> {
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_is_supported_image() {
        let scanner = ImageScanner::new(None);

        assert!(scanner.is_supported_image(Path::new("test.png")));
        assert!(scanner.is_supported_image(Path::new("test.PNG")));
        assert!(scanner.is_supported_image(Path::new("test.jpg")));
        assert!(scanner.is_supported_image(Path::new("test.jpeg")));
        assert!(scanner.is_supported_image(Path::new("test.gif")));
        assert!(scanner.is_supported_image(Path::new("test.bmp")));
        assert!(scanner.is_supported_image(Path::new("test.webp")));
        assert!(!scanner.is_supported_image(Path::new("test.txt")));
        assert!(!scanner.is_supported_image(Path::new("test.pdf")));
    }

    #[test]
    fn test_scan_no_recursion() {
        let temp = tempdir().unwrap();
        let base = temp.path();

        // ファイルを作成
        File::create(base.join("image1.png")).unwrap();
        File::create(base.join("image2.jpg")).unwrap();
        File::create(base.join("text.txt")).unwrap();

        // サブディレクトリを作成
        fs::create_dir(base.join("subdir")).unwrap();
        File::create(base.join("subdir/image3.png")).unwrap();

        let scanner = ImageScanner::new(None);
        let files = scanner.scan(base);

        // 直下の画像ファイルのみ
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.ends_with("image1.png")));
        assert!(files.iter().any(|f| f.ends_with("image2.jpg")));
    }

    #[test]
    fn test_scan_with_recursion() {
        let temp = tempdir().unwrap();
        let base = temp.path();

        // ファイルを作成
        File::create(base.join("image1.png")).unwrap();

        // サブディレクトリを作成
        fs::create_dir(base.join("subdir")).unwrap();
        File::create(base.join("subdir/image2.png")).unwrap();

        fs::create_dir(base.join("subdir/nested")).unwrap();
        File::create(base.join("subdir/nested/image3.png")).unwrap();

        // 無制限再帰
        let scanner = ImageScanner::new(Some(0));
        let files = scanner.scan(base);
        assert_eq!(files.len(), 3);

        // 深さ1まで
        let scanner = ImageScanner::new(Some(1));
        let files = scanner.scan(base);
        assert_eq!(files.len(), 2);
    }
}

