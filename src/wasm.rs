//! WASM エントリポイント
//!
//! ブラウザ上でクロマキー処理を実行するためのWASMバインディング

use std::io::Cursor;

use image::{DynamicImage, GenericImageView, ImageFormat};
use wasm_bindgen::prelude::*;

use crate::color::Rgb;
use crate::config::ProcessConfig;
use crate::pipeline::ChromaPipeline;

/// パニック時のスタックトレースをconsoleに出力
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// 処理パラメータ
#[wasm_bindgen]
pub struct WasmProcessParams {
    color: String,
    tolerance: f32,
    feather: u32,
    despill: f32,
    erode: u32,
    dilate: u32,
}

#[wasm_bindgen]
impl WasmProcessParams {
    /// 新しいパラメータを作成（デフォルト値）
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            color: "lime".to_string(),
            tolerance: 0.3,
            feather: 5,
            despill: 0.7,
            erode: 0,
            dilate: 1,
        }
    }

    /// 色を設定
    #[wasm_bindgen(js_name = setColor)]
    pub fn set_color(&mut self, color: String) {
        self.color = color;
    }

    /// 色を取得
    #[wasm_bindgen(js_name = getColor)]
    pub fn get_color(&self) -> String {
        self.color.clone()
    }

    /// 許容範囲を設定
    #[wasm_bindgen(js_name = setTolerance)]
    pub fn set_tolerance(&mut self, v: f32) {
        self.tolerance = v.clamp(0.0, 1.0);
    }

    /// 許容範囲を取得
    #[wasm_bindgen(js_name = getTolerance)]
    pub fn get_tolerance(&self) -> f32 {
        self.tolerance
    }

    /// フェザリング量を設定
    #[wasm_bindgen(js_name = setFeather)]
    pub fn set_feather(&mut self, v: u32) {
        self.feather = v.min(50);
    }

    /// フェザリング量を取得
    #[wasm_bindgen(js_name = getFeather)]
    pub fn get_feather(&self) -> u32 {
        self.feather
    }

    /// デスピル強度を設定
    #[wasm_bindgen(js_name = setDespill)]
    pub fn set_despill(&mut self, v: f32) {
        self.despill = v.clamp(0.0, 1.0);
    }

    /// デスピル強度を取得
    #[wasm_bindgen(js_name = getDespill)]
    pub fn get_despill(&self) -> f32 {
        self.despill
    }

    /// 収縮回数を設定
    #[wasm_bindgen(js_name = setErode)]
    pub fn set_erode(&mut self, v: u32) {
        self.erode = v.min(10);
    }

    /// 収縮回数を取得
    #[wasm_bindgen(js_name = getErode)]
    pub fn get_erode(&self) -> u32 {
        self.erode
    }

    /// 膨張回数を設定
    #[wasm_bindgen(js_name = setDilate)]
    pub fn set_dilate(&mut self, v: u32) {
        self.dilate = v.min(10);
    }

    /// 膨張回数を取得
    #[wasm_bindgen(js_name = getDilate)]
    pub fn get_dilate(&self) -> u32 {
        self.dilate
    }

    /// パラメータをリセット
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.color = "lime".to_string();
        self.tolerance = 0.3;
        self.feather = 5;
        self.despill = 0.7;
        self.erode = 0;
        self.dilate = 1;
    }
}

impl Default for WasmProcessParams {
    fn default() -> Self {
        Self::new()
    }
}

/// 画像情報
#[wasm_bindgen]
pub struct ImageInfo {
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl ImageInfo {
    /// 幅を取得
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// 高さを取得
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }
}

/// 画像をクロマキー処理
///
/// # Arguments
/// * `image_data` - 入力画像データ（PNG/JPEG/GIF/WebPなど）
/// * `params` - 処理パラメータ
///
/// # Returns
/// 処理後のPNG画像データ
#[wasm_bindgen(js_name = processImage)]
pub fn process_image(image_data: &[u8], params: &WasmProcessParams) -> Result<Vec<u8>, JsValue> {
    // 画像を読み込み
    let image = load_image(image_data)?;

    // 処理設定を構築
    let config = build_config(params)?;

    // 処理実行
    let pipeline = ChromaPipeline::new(config);
    let result = pipeline.process(&image.to_rgba8());

    // PNGとしてエンコード
    encode_to_png(&result)
}

/// プレビュー画像を生成（縮小版）
///
/// # Arguments
/// * `image_data` - 入力画像データ
/// * `params` - 処理パラメータ
/// * `max_size` - 最大サイズ（幅または高さ）
///
/// # Returns
/// 処理後のPNG画像データ（縮小版）
#[wasm_bindgen(js_name = processPreview)]
pub fn process_preview(
    image_data: &[u8],
    params: &WasmProcessParams,
    max_size: u32,
) -> Result<Vec<u8>, JsValue> {
    // 画像を読み込み
    let image = load_image(image_data)?;

    // リサイズ
    let image = resize_image(image, max_size);

    // 処理設定を構築
    let config = build_config(params)?;

    // 処理実行
    let pipeline = ChromaPipeline::new(config);
    let result = pipeline.process(&image.to_rgba8());

    // PNGとしてエンコード
    encode_to_png(&result)
}

/// 画像情報を取得
///
/// # Arguments
/// * `image_data` - 入力画像データ
///
/// # Returns
/// 画像情報（幅、高さ）
#[wasm_bindgen(js_name = getImageInfo)]
pub fn get_image_info(image_data: &[u8]) -> Result<ImageInfo, JsValue> {
    let image = load_image(image_data)?;
    let (width, height) = image.dimensions();
    Ok(ImageInfo { width, height })
}

/// バージョン情報を取得
#[wasm_bindgen(js_name = getVersion)]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 利用可能な色名のリストを取得
#[wasm_bindgen(js_name = getAvailableColors)]
pub fn get_available_colors() -> Vec<JsValue> {
    vec![
        "lime", "green", "blue", "magenta", "cyan", "red", "yellow", "white", "black",
    ]
    .into_iter()
    .map(|s| JsValue::from_str(s))
    .collect()
}

// === ヘルパー関数 ===

/// 画像を読み込み
fn load_image(data: &[u8]) -> Result<DynamicImage, JsValue> {
    image::load_from_memory(data).map_err(|e| JsValue::from_str(&format!("Failed to load image: {}", e)))
}

/// 処理設定を構築
fn build_config(params: &WasmProcessParams) -> Result<ProcessConfig, JsValue> {
    let chroma_color = Rgb::from_color_spec(&params.color)
        .map_err(|e| JsValue::from_str(&format!("Invalid color: {}", e)))?;

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
fn encode_to_png(image: &image::RgbaImage) -> Result<Vec<u8>, JsValue> {
    let mut buf = Cursor::new(Vec::new());
    image
        .write_to(&mut buf, ImageFormat::Png)
        .map_err(|e| JsValue::from_str(&format!("Failed to encode PNG: {}", e)))?;
    Ok(buf.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_params_default() {
        let params = WasmProcessParams::new();
        assert_eq!(params.get_color(), "lime");
        assert!((params.get_tolerance() - 0.3).abs() < 0.001);
        assert_eq!(params.get_feather(), 5);
    }

    #[test]
    fn test_wasm_params_reset() {
        let mut params = WasmProcessParams::new();
        params.set_tolerance(0.8);
        params.set_feather(20);
        params.reset();
        assert!((params.get_tolerance() - 0.3).abs() < 0.001);
        assert_eq!(params.get_feather(), 5);
    }

    #[test]
    fn test_wasm_params_clamp() {
        let mut params = WasmProcessParams::new();
        params.set_tolerance(1.5);
        assert!((params.get_tolerance() - 1.0).abs() < 0.001);

        params.set_tolerance(-0.5);
        assert!((params.get_tolerance() - 0.0).abs() < 0.001);

        params.set_feather(100);
        assert_eq!(params.get_feather(), 50);
    }
}

