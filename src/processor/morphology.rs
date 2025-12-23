//! モルフォロジー演算（最適化版）
//!
//! 改善案A: Rayon並列処理（parallel feature有効時）
//! 改善案C: ダブルバッファリング（メモリ効率改善）
//! 改善案D: バッファ直接操作

use image::GrayImage;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// 3x3カーネルでの収縮処理
/// ノイズ除去に使用
///
/// 改善案C: ダブルバッファリングで不要なメモリ割り当てを削減
pub fn erode(mask: &GrayImage, iterations: u32) -> GrayImage {
    if iterations == 0 {
        return mask.clone();
    }

    let (width, height) = mask.dimensions();

    // ダブルバッファリング（改善案C）
    let mut buf_a = mask.as_raw().clone();
    let mut buf_b = vec![0u8; (width * height) as usize];

    for i in 0..iterations {
        if i % 2 == 0 {
            erode_into(&buf_a, &mut buf_b, width, height);
        } else {
            erode_into(&buf_b, &mut buf_a, width, height);
        }
    }

    let final_data = if iterations % 2 == 0 { buf_a } else { buf_b };
    GrayImage::from_raw(width, height, final_data).expect("Failed to create image")
}

/// 3x3カーネルでの膨張処理
/// エッジ拡張に使用
///
/// 改善案C: ダブルバッファリングで不要なメモリ割り当てを削減
pub fn dilate(mask: &GrayImage, iterations: u32) -> GrayImage {
    if iterations == 0 {
        return mask.clone();
    }

    let (width, height) = mask.dimensions();

    // ダブルバッファリング（改善案C）
    let mut buf_a = mask.as_raw().clone();
    let mut buf_b = vec![0u8; (width * height) as usize];

    for i in 0..iterations {
        if i % 2 == 0 {
            dilate_into(&buf_a, &mut buf_b, width, height);
        } else {
            dilate_into(&buf_b, &mut buf_a, width, height);
        }
    }

    let final_data = if iterations % 2 == 0 { buf_a } else { buf_b };
    GrayImage::from_raw(width, height, final_data).expect("Failed to create image")
}

/// 1回の収縮処理（改善案A: 並列化、改善案D: バッファ直接操作）
fn erode_into(src: &[u8], dst: &mut [u8], width: u32, height: u32) {
    let w = width as usize;
    let h = height as usize;

    #[cfg(feature = "parallel")]
    {
        // 並列処理（改善案A）
        dst.par_chunks_mut(w)
            .enumerate()
            .for_each(|(y, row)| {
                erode_row(src, row, y, w, h);
            });
    }

    #[cfg(not(feature = "parallel"))]
    {
        // シーケンシャル処理
        for y in 0..h {
            let row = &mut dst[y * w..(y + 1) * w];
            erode_row(src, row, y, w, h);
        }
    }
}

/// 1行の収縮処理
#[inline]
fn erode_row(src: &[u8], row: &mut [u8], y: usize, w: usize, h: usize) {
    for x in 0..w {
        // 3x3カーネル内の最小値を取得
        let mut min_val = 255u8;

        for dy in 0..3 {
            let ny = y.saturating_add(dy).saturating_sub(1);
            if ny >= h {
                continue;
            }

            for dx in 0..3 {
                let nx = x.saturating_add(dx).saturating_sub(1);
                if nx >= w {
                    continue;
                }

                let val = src[ny * w + nx];
                min_val = min_val.min(val);
            }
        }

        row[x] = min_val;
    }
}

/// 1回の膨張処理（改善案A: 並列化、改善案D: バッファ直接操作）
fn dilate_into(src: &[u8], dst: &mut [u8], width: u32, height: u32) {
    let w = width as usize;
    let h = height as usize;

    #[cfg(feature = "parallel")]
    {
        // 並列処理（改善案A）
        dst.par_chunks_mut(w)
            .enumerate()
            .for_each(|(y, row)| {
                dilate_row(src, row, y, w, h);
            });
    }

    #[cfg(not(feature = "parallel"))]
    {
        // シーケンシャル処理
        for y in 0..h {
            let row = &mut dst[y * w..(y + 1) * w];
            dilate_row(src, row, y, w, h);
        }
    }
}

/// 1行の膨張処理
#[inline]
fn dilate_row(src: &[u8], row: &mut [u8], y: usize, w: usize, h: usize) {
    for x in 0..w {
        // 3x3カーネル内の最大値を取得
        let mut max_val = 0u8;

        for dy in 0..3 {
            let ny = y.saturating_add(dy).saturating_sub(1);
            if ny >= h {
                continue;
            }

            for dx in 0..3 {
                let nx = x.saturating_add(dx).saturating_sub(1);
                if nx >= w {
                    continue;
                }

                let val = src[ny * w + nx];
                max_val = max_val.max(val);
            }
        }

        row[x] = max_val;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_erode_removes_small_noise() {
        // 単一の白いピクセル（ノイズ）は収縮で消える
        let mut mask = GrayImage::new(5, 5);
        mask.put_pixel(2, 2, image::Luma([255]));

        let eroded = erode(&mask, 1);
        assert_eq!(eroded.get_pixel(2, 2).0[0], 0);
    }

    #[test]
    fn test_dilate_expands() {
        // 単一の白いピクセルが膨張で広がる
        let mut mask = GrayImage::new(5, 5);
        mask.put_pixel(2, 2, image::Luma([255]));

        let dilated = dilate(&mask, 1);

        // 中心と周囲8ピクセルが白になる
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let x = (2 + dx) as u32;
                let y = (2 + dy) as u32;
                assert_eq!(dilated.get_pixel(x, y).0[0], 255);
            }
        }
    }

    #[test]
    fn test_multiple_iterations() {
        let mut mask = GrayImage::new(7, 7);
        mask.put_pixel(3, 3, image::Luma([255]));

        // 2回膨張すると2ピクセル分広がる
        let dilated = dilate(&mask, 2);
        assert_eq!(dilated.get_pixel(1, 3).0[0], 255);
        assert_eq!(dilated.get_pixel(5, 3).0[0], 255);
    }

    #[test]
    fn test_zero_iterations() {
        let mut mask = GrayImage::new(5, 5);
        mask.put_pixel(2, 2, image::Luma([255]));

        // 0回ならそのまま
        let result = erode(&mask, 0);
        assert_eq!(result.get_pixel(2, 2).0[0], 255);

        let result = dilate(&mask, 0);
        assert_eq!(result.get_pixel(2, 2).0[0], 255);
    }

    #[test]
    fn test_large_image() {
        // 並列処理のテスト用大きな画像
        let mask = GrayImage::from_fn(1000, 1000, |x, _| {
            if x < 500 {
                image::Luma([255])
            } else {
                image::Luma([0])
            }
        });

        let dilated = dilate(&mask, 1);
        // 膨張により境界が広がる
        assert_eq!(dilated.get_pixel(500, 500).0[0], 255);
    }
}
