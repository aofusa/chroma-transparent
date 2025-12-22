//! HSV変換用ルックアップテーブル（改善案F）
//!
//! RGB→HSV変換を高速化するため、量子化されたルックアップテーブルを使用

use super::{rgb_to_hsv, Hsv, Rgb};

/// 量子化レベル（32段階 = 256/8）
const QUANT_LEVELS: usize = 32;
const QUANT_SHIFT: usize = 3; // 256 / 32 = 8 = 2^3

/// HSV変換用ルックアップテーブル
///
/// 32x32x32 = 32,768エントリで約400KBのメモリを使用
pub struct HsvLut {
    table: Vec<Hsv>,
}

impl HsvLut {
    /// 新しいLUTを作成
    ///
    /// 初期化時に全エントリを事前計算
    pub fn new() -> Self {
        let capacity = QUANT_LEVELS * QUANT_LEVELS * QUANT_LEVELS;
        let mut table = Vec::with_capacity(capacity);

        for r in 0..QUANT_LEVELS {
            for g in 0..QUANT_LEVELS {
                for b in 0..QUANT_LEVELS {
                    // 各量子化レベルの中央値を使用
                    let rgb = Rgb::new(
                        ((r << QUANT_SHIFT) + 4) as u8,
                        ((g << QUANT_SHIFT) + 4) as u8,
                        ((b << QUANT_SHIFT) + 4) as u8,
                    );
                    table.push(rgb_to_hsv(&rgb));
                }
            }
        }

        Self { table }
    }

    /// RGB値からHSVを取得（高速版）
    ///
    /// 量子化による誤差があるが、高速
    #[inline]
    pub fn get(&self, rgb: &Rgb) -> Hsv {
        let idx = self.index(rgb.r, rgb.g, rgb.b);
        self.table[idx]
    }

    /// インデックスを計算
    #[inline]
    fn index(&self, r: u8, g: u8, b: u8) -> usize {
        let r_idx = (r as usize) >> QUANT_SHIFT;
        let g_idx = (g as usize) >> QUANT_SHIFT;
        let b_idx = (b as usize) >> QUANT_SHIFT;
        (r_idx * QUANT_LEVELS * QUANT_LEVELS) + (g_idx * QUANT_LEVELS) + b_idx
    }
}

impl Default for HsvLut {
    fn default() -> Self {
        Self::new()
    }
}

/// RGB空間での距離計算（改善案B）
///
/// HSV変換より高速だが、人間の知覚との乖離がある
#[inline]
pub fn rgb_distance_squared(a: &Rgb, b: &Rgb) -> u32 {
    let dr = a.r as i32 - b.r as i32;
    let dg = a.g as i32 - b.g as i32;
    let db = a.b as i32 - b.b as i32;
    (dr * dr + dg * dg + db * db) as u32
}

/// RGB距離を正規化（0.0 - 1.0）
#[inline]
pub fn rgb_distance_normalized(a: &Rgb, b: &Rgb) -> f32 {
    let dist_sq = rgb_distance_squared(a, b);
    // 最大距離: sqrt(255^2 * 3) ≈ 441.67
    // 最大距離の二乗: 255^2 * 3 = 195075
    (dist_sq as f32 / 195075.0).sqrt()
}

/// 高速事前フィルタリング（改善案B）
///
/// RGB距離で明らかに異なる色を事前に除外
#[inline]
pub fn is_obviously_different(pixel: &Rgb, target: &Rgb, threshold_squared: u32) -> bool {
    rgb_distance_squared(pixel, target) > threshold_squared
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lut_basic() {
        let lut = HsvLut::new();

        // 緑色
        let green = Rgb::new(0, 255, 0);
        let hsv = lut.get(&green);
        // 量子化誤差を許容
        assert!((hsv.h - 120.0).abs() < 10.0);
        assert!(hsv.s > 0.9);
        assert!(hsv.v > 0.9);
    }

    #[test]
    fn test_lut_size() {
        let lut = HsvLut::new();
        assert_eq!(lut.table.len(), 32 * 32 * 32);
    }

    #[test]
    fn test_rgb_distance() {
        let a = Rgb::new(0, 255, 0);
        let b = Rgb::new(0, 255, 0);
        assert_eq!(rgb_distance_squared(&a, &b), 0);

        let c = Rgb::new(255, 0, 0);
        let dist = rgb_distance_squared(&a, &c);
        assert!(dist > 0);
    }

    #[test]
    fn test_rgb_distance_normalized() {
        let a = Rgb::new(0, 0, 0);
        let b = Rgb::new(255, 255, 255);
        let dist = rgb_distance_normalized(&a, &b);
        assert!((dist - 1.0).abs() < 0.01);
    }
}

