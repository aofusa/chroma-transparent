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
    
    // 新規フィールド
    color_space: String,
    bilateral_enabled: bool,
    bilateral_spatial_sigma: f32,
    bilateral_color_sigma: f32,
    bilateral_radius: u32,
    multiscale_enabled: bool,
    multiscale_levels: u32,
    multiscale_scale_factor: f32,
    edge_optimization_enabled: bool,
    edge_threshold: f32,
    edge_smoothness: f32,
    shadow_removal_enabled: bool,
    shadow_threshold: f32,
    shadow_removal_strength: f32,
    sharpen_enabled: bool,
    sharpen_amount: f32,
    sharpen_radius: f32,
    sharpen_threshold: f32,
    
    // 新規実装機能
    adaptive_tolerance_enabled: bool,
    adaptive_tolerance_grid_w: u32,
    adaptive_tolerance_grid_h: u32,
    adaptive_tolerance_sensitivity: f32,
    
    edge_detection_method: String,
    canny_low_threshold: f32,
    canny_high_threshold: f32,
    canny_gaussian_sigma: f32,
    
    despill_method: String,
    
    thin_line_detection_enabled: bool,
    thin_line_sensitivity: f32,
    thin_line_threshold: f32,
    
    auto_params_enabled: bool,
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
            
            // 新規フィールドのデフォルト値
            color_space: "hsv".to_string(),
            bilateral_enabled: false,
            bilateral_spatial_sigma: 5.0,
            bilateral_color_sigma: 50.0,
            bilateral_radius: 5,
            multiscale_enabled: false,
            multiscale_levels: 3,
            multiscale_scale_factor: 0.5,
            edge_optimization_enabled: false,
            edge_threshold: 0.1,
            edge_smoothness: 0.5,
            shadow_removal_enabled: false,
            shadow_threshold: 0.3,
            shadow_removal_strength: 0.7,
            sharpen_enabled: false,
            sharpen_amount: 0.5,
            sharpen_radius: 1.0,
            sharpen_threshold: 0.0,
            
            adaptive_tolerance_enabled: false,
            adaptive_tolerance_grid_w: 8,
            adaptive_tolerance_grid_h: 8,
            adaptive_tolerance_sensitivity: 1.0,
            
            edge_detection_method: "sobel".to_string(),
            canny_low_threshold: 0.1,
            canny_high_threshold: 0.3,
            canny_gaussian_sigma: 1.0,
            
            despill_method: "basic".to_string(),
            
            thin_line_detection_enabled: false,
            thin_line_sensitivity: 0.5,
            thin_line_threshold: 0.3,
            
            auto_params_enabled: false,
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
        
        // 新規フィールドのリセット
        self.color_space = "hsv".to_string();
        self.bilateral_enabled = false;
        self.bilateral_spatial_sigma = 5.0;
        self.bilateral_color_sigma = 50.0;
        self.bilateral_radius = 5;
        self.multiscale_enabled = false;
        self.multiscale_levels = 3;
        self.multiscale_scale_factor = 0.5;
        self.edge_optimization_enabled = false;
        self.edge_threshold = 0.1;
        self.edge_smoothness = 0.5;
        self.shadow_removal_enabled = false;
        self.shadow_threshold = 0.3;
        self.shadow_removal_strength = 0.7;
        self.sharpen_enabled = false;
        self.sharpen_amount = 0.5;
        self.sharpen_radius = 1.0;
        self.sharpen_threshold = 0.0;
        
        self.adaptive_tolerance_enabled = false;
        self.adaptive_tolerance_grid_w = 8;
        self.adaptive_tolerance_grid_h = 8;
        self.adaptive_tolerance_sensitivity = 1.0;
        
        self.edge_detection_method = "sobel".to_string();
        self.canny_low_threshold = 0.1;
        self.canny_high_threshold = 0.3;
        self.canny_gaussian_sigma = 1.0;
        
        self.despill_method = "basic".to_string();
        
        self.thin_line_detection_enabled = false;
        self.thin_line_sensitivity = 0.5;
        self.thin_line_threshold = 0.3;
        
        self.auto_params_enabled = false;
    }
    
    // 新規フィールドのgetter/setter
    #[wasm_bindgen(js_name = setColorSpace)]
    pub fn set_color_space(&mut self, v: String) {
        self.color_space = v;
    }
    
    #[wasm_bindgen(js_name = getColorSpace)]
    pub fn get_color_space(&self) -> String {
        self.color_space.clone()
    }
    
    #[wasm_bindgen(js_name = setBilateralEnabled)]
    pub fn set_bilateral_enabled(&mut self, v: bool) {
        self.bilateral_enabled = v;
    }
    
    #[wasm_bindgen(js_name = getBilateralEnabled)]
    pub fn get_bilateral_enabled(&self) -> bool {
        self.bilateral_enabled
    }
    
    #[wasm_bindgen(js_name = setBilateralSpatialSigma)]
    pub fn set_bilateral_spatial_sigma(&mut self, v: f32) {
        self.bilateral_spatial_sigma = v.clamp(1.0, 20.0);
    }
    
    #[wasm_bindgen(js_name = getBilateralSpatialSigma)]
    pub fn get_bilateral_spatial_sigma(&self) -> f32 {
        self.bilateral_spatial_sigma
    }
    
    #[wasm_bindgen(js_name = setBilateralColorSigma)]
    pub fn set_bilateral_color_sigma(&mut self, v: f32) {
        self.bilateral_color_sigma = v.clamp(10.0, 100.0);
    }
    
    #[wasm_bindgen(js_name = getBilateralColorSigma)]
    pub fn get_bilateral_color_sigma(&self) -> f32 {
        self.bilateral_color_sigma
    }
    
    #[wasm_bindgen(js_name = setBilateralRadius)]
    pub fn set_bilateral_radius(&mut self, v: u32) {
        self.bilateral_radius = v.min(10);
    }
    
    #[wasm_bindgen(js_name = getBilateralRadius)]
    pub fn get_bilateral_radius(&self) -> u32 {
        self.bilateral_radius
    }
    
    #[wasm_bindgen(js_name = setMultiscaleEnabled)]
    pub fn set_multiscale_enabled(&mut self, v: bool) {
        self.multiscale_enabled = v;
    }
    
    #[wasm_bindgen(js_name = getMultiscaleEnabled)]
    pub fn get_multiscale_enabled(&self) -> bool {
        self.multiscale_enabled
    }
    
    #[wasm_bindgen(js_name = setMultiscaleLevels)]
    pub fn set_multiscale_levels(&mut self, v: u32) {
        self.multiscale_levels = v.min(5).max(1);
    }
    
    #[wasm_bindgen(js_name = getMultiscaleLevels)]
    pub fn get_multiscale_levels(&self) -> u32 {
        self.multiscale_levels
    }
    
    #[wasm_bindgen(js_name = setMultiscaleScaleFactor)]
    pub fn set_multiscale_scale_factor(&mut self, v: f32) {
        self.multiscale_scale_factor = v.clamp(0.25, 0.75);
    }
    
    #[wasm_bindgen(js_name = getMultiscaleScaleFactor)]
    pub fn get_multiscale_scale_factor(&self) -> f32 {
        self.multiscale_scale_factor
    }
    
    #[wasm_bindgen(js_name = setEdgeOptimizationEnabled)]
    pub fn set_edge_optimization_enabled(&mut self, v: bool) {
        self.edge_optimization_enabled = v;
    }
    
    #[wasm_bindgen(js_name = getEdgeOptimizationEnabled)]
    pub fn get_edge_optimization_enabled(&self) -> bool {
        self.edge_optimization_enabled
    }
    
    #[wasm_bindgen(js_name = setEdgeThreshold)]
    pub fn set_edge_threshold(&mut self, v: f32) {
        self.edge_threshold = v.clamp(0.0, 1.0);
    }
    
    #[wasm_bindgen(js_name = getEdgeThreshold)]
    pub fn get_edge_threshold(&self) -> f32 {
        self.edge_threshold
    }
    
    #[wasm_bindgen(js_name = setEdgeSmoothness)]
    pub fn set_edge_smoothness(&mut self, v: f32) {
        self.edge_smoothness = v.clamp(0.0, 1.0);
    }
    
    #[wasm_bindgen(js_name = getEdgeSmoothness)]
    pub fn get_edge_smoothness(&self) -> f32 {
        self.edge_smoothness
    }
    
    #[wasm_bindgen(js_name = setShadowRemovalEnabled)]
    pub fn set_shadow_removal_enabled(&mut self, v: bool) {
        self.shadow_removal_enabled = v;
    }
    
    #[wasm_bindgen(js_name = getShadowRemovalEnabled)]
    pub fn get_shadow_removal_enabled(&self) -> bool {
        self.shadow_removal_enabled
    }
    
    #[wasm_bindgen(js_name = setShadowThreshold)]
    pub fn set_shadow_threshold(&mut self, v: f32) {
        self.shadow_threshold = v.clamp(0.0, 1.0);
    }
    
    #[wasm_bindgen(js_name = getShadowThreshold)]
    pub fn get_shadow_threshold(&self) -> f32 {
        self.shadow_threshold
    }
    
    #[wasm_bindgen(js_name = setShadowRemovalStrength)]
    pub fn set_shadow_removal_strength(&mut self, v: f32) {
        self.shadow_removal_strength = v.clamp(0.0, 1.0);
    }
    
    #[wasm_bindgen(js_name = getShadowRemovalStrength)]
    pub fn get_shadow_removal_strength(&self) -> f32 {
        self.shadow_removal_strength
    }
    
    #[wasm_bindgen(js_name = setSharpenEnabled)]
    pub fn set_sharpen_enabled(&mut self, v: bool) {
        self.sharpen_enabled = v;
    }
    
    #[wasm_bindgen(js_name = getSharpenEnabled)]
    pub fn get_sharpen_enabled(&self) -> bool {
        self.sharpen_enabled
    }
    
    #[wasm_bindgen(js_name = setSharpenAmount)]
    pub fn set_sharpen_amount(&mut self, v: f32) {
        self.sharpen_amount = v.clamp(0.0, 2.0);
    }
    
    #[wasm_bindgen(js_name = getSharpenAmount)]
    pub fn get_sharpen_amount(&self) -> f32 {
        self.sharpen_amount
    }
    
    #[wasm_bindgen(js_name = setSharpenRadius)]
    pub fn set_sharpen_radius(&mut self, v: f32) {
        self.sharpen_radius = v.clamp(0.1, 5.0);
    }
    
    #[wasm_bindgen(js_name = getSharpenRadius)]
    pub fn get_sharpen_radius(&self) -> f32 {
        self.sharpen_radius
    }
    
    #[wasm_bindgen(js_name = setSharpenThreshold)]
    pub fn set_sharpen_threshold(&mut self, v: f32) {
        self.sharpen_threshold = v.clamp(0.0, 1.0);
    }
    
    #[wasm_bindgen(js_name = getSharpenThreshold)]
    pub fn get_sharpen_threshold(&self) -> f32 {
        self.sharpen_threshold
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
    use crate::color::ColorSpace;
    
    let chroma_color = Rgb::from_color_spec(&params.color)
        .map_err(|e| JsValue::from_str(&format!("Invalid color: {}", e)))?;

    let color_space = ColorSpace::from_str(&params.color_space)
        .map_err(|e| JsValue::from_str(&format!("Invalid color space: {}", e)))?;

    let mut config = ProcessConfig::default();
    config.chroma_color = chroma_color;
    config.tolerance = params.tolerance;
    config.feather_amount = params.feather;
    config.despill_strength = params.despill;
    config.erode_iterations = params.erode;
    config.dilate_iterations = params.dilate;
    
    // 新規フィールド
    config.color_space = color_space;
    config.bilateral_enabled = params.bilateral_enabled;
    config.bilateral_spatial_sigma = params.bilateral_spatial_sigma;
    config.bilateral_color_sigma = params.bilateral_color_sigma;
    config.bilateral_radius = params.bilateral_radius;
    config.multiscale_enabled = params.multiscale_enabled;
    config.multiscale_levels = params.multiscale_levels;
    config.multiscale_scale_factor = params.multiscale_scale_factor;
    config.edge_optimization_enabled = params.edge_optimization_enabled;
    config.edge_threshold = params.edge_threshold;
    config.edge_smoothness = params.edge_smoothness;
    config.shadow_removal_enabled = params.shadow_removal_enabled;
    config.shadow_threshold = params.shadow_threshold;
    config.shadow_removal_strength = params.shadow_removal_strength;
    config.sharpen_enabled = params.sharpen_enabled;
    config.sharpen_amount = params.sharpen_amount;
    config.sharpen_radius = params.sharpen_radius;
    config.sharpen_threshold = params.sharpen_threshold;
    
    // 新規実装機能
    config.adaptive_tolerance_enabled = params.adaptive_tolerance_enabled;
    config.adaptive_tolerance_grid_w = params.adaptive_tolerance_grid_w;
    config.adaptive_tolerance_grid_h = params.adaptive_tolerance_grid_h;
    config.adaptive_tolerance_sensitivity = params.adaptive_tolerance_sensitivity;
    
    config.edge_detection_method = match params.edge_detection_method.as_str() {
        "canny" => crate::processor::EdgeDetectionMethod::Canny,
        _ => crate::processor::EdgeDetectionMethod::Sobel,
    };
    config.canny_low_threshold = params.canny_low_threshold;
    config.canny_high_threshold = params.canny_high_threshold;
    config.canny_gaussian_sigma = params.canny_gaussian_sigma;
    
    config.despill_method = match params.despill_method.as_str() {
        "advanced" => crate::processor::DespillMethod::Advanced,
        _ => crate::processor::DespillMethod::Basic,
    };
    
    config.thin_line_detection_enabled = params.thin_line_detection_enabled;
    config.thin_line_sensitivity = params.thin_line_sensitivity;
    config.thin_line_threshold = params.thin_line_threshold;
    
    config.auto_params_enabled = params.auto_params_enabled;
    
    Ok(config)
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

