//! Despill処理（最適化版）
//!
//! 改善案A: Rayon並列処理（parallel feature有効時）
//! 改善案D: バッファ直接操作
//! 改善案G: インプレース処理

use image::{GrayImage, RgbaImage};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use crate::color::{rgb_to_yuv, Rgb, Yuv};

/// デスピル方法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DespillMethod {
    Basic,    // 既存の基本的な方法
    Advanced, // YUV色空間を使用した高度な方法
}

/// クロマスピルを除去
///
/// グリーンバックの反射（グリーンスピル）をエッジ周辺から除去
///
/// # Arguments
/// * `image` - 処理対象の画像（インプレース変更）
/// * `mask` - クロマキーマスク（エッジ周辺を特定）
/// * `strength` - Despill強度 (0.0 - 1.0)
/// * `chroma_color` - クロマキー対象色
/// * `method` - デスピル方法（Basic/Advanced）
///
/// # 改善案G: インプレース処理
/// 画像を新規作成せず、既存バッファを直接変更
pub fn despill(
    image: &mut RgbaImage,
    mask: &GrayImage,
    strength: f32,
    chroma_color: &Rgb,
    method: DespillMethod,
) {
    if strength <= 0.0 {
        return;
    }

    let strength = strength.clamp(0.0, 1.0);
    let (width, _height) = image.dimensions();
    let w = width as usize;

    // クロマ色の主成分を特定
    let chroma_channel = determine_chroma_channel(chroma_color);

    // バッファを取得
    let raw = image.as_mut();
    let mask_raw = mask.as_raw();

    match method {
        DespillMethod::Basic => {
            #[cfg(feature = "parallel")]
            {
                raw.par_chunks_mut(w * 4)
                    .enumerate()
                    .for_each(|(y, row)| {
                        despill_row(row, y, w, mask_raw, strength, chroma_channel);
                    });
            }

            #[cfg(not(feature = "parallel"))]
            {
                for y in 0..(raw.len() / (w * 4)) {
                    let row = &mut raw[y * w * 4..(y + 1) * w * 4];
                    despill_row(row, y, w, mask_raw, strength, chroma_channel);
                }
            }
        }
        DespillMethod::Advanced => {
            let target_yuv = rgb_to_yuv(chroma_color);
            #[cfg(feature = "parallel")]
            {
                raw.par_chunks_mut(w * 4)
                    .enumerate()
                    .for_each(|(y, row)| {
                        despill_row_advanced(row, y, w, mask_raw, strength, chroma_color, &target_yuv);
                    });
            }

            #[cfg(not(feature = "parallel"))]
            {
                for y in 0..(raw.len() / (w * 4)) {
                    let row = &mut raw[y * w * 4..(y + 1) * w * 4];
                    despill_row_advanced(row, y, w, mask_raw, strength, chroma_color, &target_yuv);
                }
            }
        }
    }
}

/// 1行のDespill処理
fn despill_row(row: &mut [u8], y: usize, w: usize, mask_raw: &[u8], strength: f32, chroma_channel: ChromaChannel) {
    for x in 0..w {
        let mask_val = mask_raw[y * w + x];

        // マスク値に基づいてDespill強度を調整
        // マスク値が低い（エッジ周辺）ほど強くDespill
        let edge_factor = 1.0 - (mask_val as f32 / 255.0);
        if edge_factor <= 0.01 {
            continue;
        }

        let idx = x * 4;
        let r = row[idx];
        let g = row[idx + 1];
        let b = row[idx + 2];

        let (new_r, new_g, new_b) = match chroma_channel {
            ChromaChannel::Green => despill_green(r, g, b, strength * edge_factor),
            ChromaChannel::Blue => despill_blue(r, g, b, strength * edge_factor),
            ChromaChannel::Red => despill_red(r, g, b, strength * edge_factor),
        };

        // インプレース更新（改善案G）
        row[idx] = new_r;
        row[idx + 1] = new_g;
        row[idx + 2] = new_b;
    }
}

/// クロマ色の主成分を判定
fn determine_chroma_channel(chroma_color: &Rgb) -> ChromaChannel {
    if chroma_color.g > chroma_color.r && chroma_color.g > chroma_color.b {
        ChromaChannel::Green
    } else if chroma_color.b > chroma_color.r && chroma_color.b > chroma_color.g {
        ChromaChannel::Blue
    } else {
        ChromaChannel::Red
    }
}

#[derive(Clone, Copy)]
enum ChromaChannel {
    Red,
    Green,
    Blue,
}

/// グリーンスピル除去
#[inline]
fn despill_green(r: u8, g: u8, b: u8, strength: f32) -> (u8, u8, u8) {
    let r_f = r as f32;
    let g_f = g as f32;
    let b_f = b as f32;

    // 緑が他のチャンネルの平均より高い場合にDespill
    let avg = (r_f + b_f) / 2.0;
    if g_f > avg {
        let excess = (g_f - avg) * strength;
        let new_g = (g_f - excess).max(0.0) as u8;
        (r, new_g, b)
    } else {
        (r, g, b)
    }
}

/// ブルースピル除去
#[inline]
fn despill_blue(r: u8, g: u8, b: u8, strength: f32) -> (u8, u8, u8) {
    let r_f = r as f32;
    let g_f = g as f32;
    let b_f = b as f32;

    let avg = (r_f + g_f) / 2.0;
    if b_f > avg {
        let excess = (b_f - avg) * strength;
        let new_b = (b_f - excess).max(0.0) as u8;
        (r, g, new_b)
    } else {
        (r, g, b)
    }
}

/// レッドスピル除去
#[inline]
fn despill_red(r: u8, g: u8, b: u8, strength: f32) -> (u8, u8, u8) {
    let r_f = r as f32;
    let g_f = g as f32;
    let b_f = b as f32;

    let avg = (g_f + b_f) / 2.0;
    if r_f > avg {
        let excess = (r_f - avg) * strength;
        let new_r = (r_f - excess).max(0.0) as u8;
        (new_r, g, b)
    } else {
        (r, g, b)
    }
}

/// 1行の高度なDespill処理（YUV色空間使用）
fn despill_row_advanced(
    row: &mut [u8],
    y: usize,
    w: usize,
    mask_raw: &[u8],
    strength: f32,
    chroma_color: &Rgb,
    target_yuv: &Yuv,
) {
    for x in 0..w {
        let mask_val = mask_raw[y * w + x];

        // マスク値に基づいてDespill強度を調整
        let edge_factor = 1.0 - (mask_val as f32 / 255.0);
        if edge_factor <= 0.01 {
            continue;
        }

        let idx = x * 4;
        let r = row[idx];
        let g = row[idx + 1];
        let b = row[idx + 2];

        let pixel_rgb = Rgb::new(r, g, b);
        let pixel_yuv = rgb_to_yuv(&pixel_rgb);

        // YUV色空間で色かぶりを検出
        let (new_r, new_g, new_b) = despill_yuv(
            &pixel_rgb,
            &pixel_yuv,
            target_yuv,
            strength * edge_factor,
        );

        // インプレース更新
        row[idx] = new_r;
        row[idx + 1] = new_g;
        row[idx + 2] = new_b;
    }
}

/// YUV色空間でデスピル処理
#[inline]
fn despill_yuv(
    pixel_rgb: &Rgb,
    pixel_yuv: &Yuv,
    target_yuv: &Yuv,
    strength: f32,
) -> (u8, u8, u8) {
    // クロマ色の主成分を特定
    let chroma_channel = if target_yuv.u.abs() > target_yuv.v.abs() {
        // U成分が大きい（青-黄軸）
        if target_yuv.u > 0.0 {
            'b' // 青
        } else {
            'y' // 黄
        }
    } else {
        // V成分が大きい（赤-緑軸）
        if target_yuv.v > 0.0 {
            'r' // 赤
        } else {
            'g' // 緑
        }
    };

    let r_f = pixel_rgb.r as f32;
    let g_f = pixel_rgb.g as f32;
    let b_f = pixel_rgb.b as f32;

    // YUV色空間での色かぶりを検出
    let u_diff = (pixel_yuv.u - target_yuv.u).abs();
    let v_diff = (pixel_yuv.v - target_yuv.v).abs();
    let chroma_similarity = 1.0 - (u_diff + v_diff).min(1.0);

    // 非線形補正
    let correction_factor = chroma_similarity * strength;
    let correction = correction_factor * correction_factor; // 非線形

    match chroma_channel {
        'g' => {
            // グリーンスピル除去
            let avg = (r_f + b_f) / 2.0;
            if g_f > avg {
                let excess = (g_f - avg) * correction;
                let new_g = (g_f - excess).max(0.0) as u8;
                (pixel_rgb.r, new_g, pixel_rgb.b)
            } else {
                (pixel_rgb.r, pixel_rgb.g, pixel_rgb.b)
            }
        }
        'b' => {
            // ブルースピル除去
            let avg = (r_f + g_f) / 2.0;
            if b_f > avg {
                let excess = (b_f - avg) * correction;
                let new_b = (b_f - excess).max(0.0) as u8;
                (pixel_rgb.r, pixel_rgb.g, new_b)
            } else {
                (pixel_rgb.r, pixel_rgb.g, pixel_rgb.b)
            }
        }
        'r' => {
            // レッドスピル除去
            let avg = (g_f + b_f) / 2.0;
            if r_f > avg {
                let excess = (r_f - avg) * correction;
                let new_r = (r_f - excess).max(0.0) as u8;
                (new_r, pixel_rgb.g, pixel_rgb.b)
            } else {
                (pixel_rgb.r, pixel_rgb.g, pixel_rgb.b)
            }
        }
        _ => (pixel_rgb.r, pixel_rgb.g, pixel_rgb.b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_despill_green() {
        let mut image = RgbaImage::new(1, 1);
        image.put_pixel(0, 0, Rgba([100, 200, 100, 255])); // 緑が強い

        let mut mask = GrayImage::new(1, 1);
        mask.put_pixel(0, 0, image::Luma([128])); // エッジ周辺

        let chroma = Rgb::new(0, 255, 0);
        despill(&mut image, &mask, 1.0, &chroma, DespillMethod::Basic);

        let pixel = image.get_pixel(0, 0);
        // 緑成分が減少
        assert!(pixel[1] < 200);
        // 他のチャンネルは変化なし
        assert_eq!(pixel[0], 100);
        assert_eq!(pixel[2], 100);
    }

    #[test]
    fn test_zero_strength() {
        let mut image = RgbaImage::new(1, 1);
        image.put_pixel(0, 0, Rgba([100, 200, 100, 255]));

        let mut mask = GrayImage::new(1, 1);
        mask.put_pixel(0, 0, image::Luma([128]));

        let chroma = Rgb::new(0, 255, 0);
        despill(&mut image, &mask, 0.0, &chroma, DespillMethod::Basic);

        // 変化なし
        let pixel = image.get_pixel(0, 0);
        assert_eq!(pixel[1], 200);
    }

    #[test]
    fn test_mask_255_no_despill() {
        let mut image = RgbaImage::new(1, 1);
        image.put_pixel(0, 0, Rgba([100, 200, 100, 255]));

        let mut mask = GrayImage::new(1, 1);
        mask.put_pixel(0, 0, image::Luma([255])); // マスク内（エッジではない）

        let chroma = Rgb::new(0, 255, 0);
        despill(&mut image, &mask, 1.0, &chroma, DespillMethod::Basic);

        // マスク値255（完全にクロマキー領域）ではDespillしない
        let pixel = image.get_pixel(0, 0);
        assert_eq!(pixel[1], 200);
    }

    #[test]
    fn test_large_image() {
        // 並列処理のテスト用大きな画像
        let mut image = RgbaImage::from_fn(500, 500, |_, _| Rgba([100, 200, 100, 255]));

        let mask = GrayImage::from_fn(500, 500, |_, _| image::Luma([128]));

        let chroma = Rgb::new(0, 255, 0);
        despill(&mut image, &mask, 0.5, &chroma, DespillMethod::Basic);

        // 全ピクセルの緑成分が減少
        for pixel in image.pixels() {
            assert!(pixel[1] < 200);
        }
    }
}
