//! 画像処理モジュール（最適化版）
//!
//! パフォーマンス改善:
//! - A: Rayon並列処理（parallel feature有効時）
//! - B: RGB距離による事前フィルタリング
//! - C: ダブルバッファリング
//! - D: バッファ直接操作
//! - E: SIMD最適化（非WASM環境）
//! - F: LUT
//! - G: インプレース処理

pub mod alpha;
pub mod bilateral;
pub mod despill;
pub mod edge_optimization;
pub mod feather;
pub mod mask;
pub mod morphology;
pub mod multiscale;
pub mod shadow;
pub mod sharpen;

#[cfg(not(target_arch = "wasm32"))]
pub mod simd;

pub use alpha::{apply_alpha, apply_alpha_inplace, create_alpha_from_mask};
pub use bilateral::bilateral_filter_alpha;
pub use despill::despill;
pub use edge_optimization::optimize_edge;
pub use feather::feather_alpha;
pub use mask::{create_chroma_mask, create_multi_chroma_mask};
pub use morphology::{dilate, erode};
pub use multiscale::create_multiscale_mask;
pub use shadow::remove_shadows;
pub use sharpen::sharpen;

