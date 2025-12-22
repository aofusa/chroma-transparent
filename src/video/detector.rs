//! ファイル種別判定モジュール

use std::path::Path;

/// ファイルの種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// 画像ファイル
    Image,
    /// 動画ファイル
    Video,
    /// ディレクトリ
    Directory,
    /// 不明
    Unknown,
}

/// 対応する画像拡張子
const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "bmp", "webp", "tiff", "tif",
];

/// 対応する動画拡張子
const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "webm", "mov", "avi", "mkv", "m4v", "wmv", "flv", "ogv",
];

/// 拡張子からファイル種別を推測
fn detect_by_extension(path: &Path) -> FileType {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match ext.as_deref() {
        Some(ext) if IMAGE_EXTENSIONS.contains(&ext) => FileType::Image,
        Some(ext) if VIDEO_EXTENSIONS.contains(&ext) => FileType::Video,
        _ => FileType::Unknown,
    }
}

/// ファイル種別を判定（ファイルシステムを確認）
pub fn detect_file_type(path: &Path) -> FileType {
    if path.is_dir() {
        return FileType::Directory;
    }

    // ファイルが存在する場合のみ、拡張子で判定
    if path.is_file() {
        return detect_by_extension(path);
    }

    // ファイルが存在しない場合でも拡張子から推測
    // （これにより、出力パスの判定にも使える）
    detect_by_extension(path)
}

/// 動画ファイルかどうか（存在確認あり）
pub fn is_video_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    matches!(detect_by_extension(path), FileType::Video)
}

/// 画像ファイルかどうか（存在確認あり）
#[allow(dead_code)]
pub fn is_image_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    matches!(detect_by_extension(path), FileType::Image)
}

/// 拡張子から動画かどうかを判定（存在確認なし）
#[allow(dead_code)]
pub fn is_video_extension(path: &Path) -> bool {
    matches!(detect_by_extension(path), FileType::Video)
}

/// 拡張子から画像かどうかを判定（存在確認なし）
#[allow(dead_code)]
pub fn is_image_extension(path: &Path) -> bool {
    matches!(detect_by_extension(path), FileType::Image)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_by_extension_image() {
        assert_eq!(detect_by_extension(Path::new("test.png")), FileType::Image);
        assert_eq!(detect_by_extension(Path::new("test.PNG")), FileType::Image);
        assert_eq!(detect_by_extension(Path::new("test.jpg")), FileType::Image);
        assert_eq!(detect_by_extension(Path::new("test.jpeg")), FileType::Image);
        assert_eq!(detect_by_extension(Path::new("test.webp")), FileType::Image);
    }

    #[test]
    fn test_detect_by_extension_video() {
        assert_eq!(detect_by_extension(Path::new("test.mp4")), FileType::Video);
        assert_eq!(detect_by_extension(Path::new("test.MP4")), FileType::Video);
        assert_eq!(detect_by_extension(Path::new("test.webm")), FileType::Video);
        assert_eq!(detect_by_extension(Path::new("test.mov")), FileType::Video);
        assert_eq!(detect_by_extension(Path::new("test.avi")), FileType::Video);
    }

    #[test]
    fn test_detect_by_extension_unknown() {
        assert_eq!(detect_by_extension(Path::new("test.txt")), FileType::Unknown);
        assert_eq!(detect_by_extension(Path::new("test.pdf")), FileType::Unknown);
        assert_eq!(detect_by_extension(Path::new("noextension")), FileType::Unknown);
    }

    #[test]
    fn test_is_video_extension() {
        assert!(is_video_extension(Path::new("test.mp4")));
        assert!(is_video_extension(Path::new("test.webm")));
        assert!(!is_video_extension(Path::new("test.png")));
        assert!(!is_video_extension(Path::new("test.txt")));
    }

    #[test]
    fn test_is_image_extension() {
        assert!(is_image_extension(Path::new("test.png")));
        assert!(is_image_extension(Path::new("test.jpg")));
        assert!(!is_image_extension(Path::new("test.mp4")));
        assert!(!is_image_extension(Path::new("test.txt")));
    }
}
