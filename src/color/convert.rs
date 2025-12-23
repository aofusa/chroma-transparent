//! 色空間変換

use super::{Hsv, Rgb};

/// LAB色構造体
#[derive(Debug, Clone, Copy)]
pub struct Lab {
    /// L*: 明度 (0.0 - 100.0)
    pub l: f32,
    /// a*: 緑-赤軸 (-128.0 - 127.0)
    pub a: f32,
    /// b*: 青-黄軸 (-128.0 - 127.0)
    pub b: f32,
}

impl Lab {
    pub fn new(l: f32, a: f32, b: f32) -> Self {
        Self { l, a, b }
    }

    /// 2つのLAB色の距離を計算（ユークリッド距離）
    pub fn distance(&self, other: &Lab) -> f32 {
        let dl = self.l - other.l;
        let da = self.a - other.a;
        let db = self.b - other.b;
        (dl * dl + da * da + db * db).sqrt()
    }

    /// 指定した許容範囲内かどうか判定
    pub fn is_within_tolerance(&self, target: &Lab, tolerance: f32) -> bool {
        // toleranceをLAB空間の距離に変換（簡易版）
        // L*は0-100、a*とb*は-128-127の範囲なので、正規化が必要
        let normalized_tolerance = tolerance * 200.0; // 大まかな正規化
        self.distance(target) < normalized_tolerance
    }
}

/// LCH色構造体
#[derive(Debug, Clone, Copy)]
pub struct Lch {
    /// L*: 明度 (0.0 - 100.0)
    pub l: f32,
    /// C*: 彩度 (0.0 - 150.0)
    pub c: f32,
    /// h*: 色相角 (0.0 - 360.0)
    pub h: f32,
}

impl Lch {
    pub fn new(l: f32, c: f32, h: f32) -> Self {
        Self { l, c, h }
    }

    /// 2つのLCH色の距離を計算
    /// 色相は円環として計算
    pub fn distance(&self, other: &Lch) -> f32 {
        let dl = self.l - other.l;
        let dc = self.c - other.c;
        
        // 色相の差（円環距離）
        let h_diff = {
            let diff = (self.h - other.h).abs();
            if diff > 180.0 { 360.0 - diff } else { diff }
        };
        
        // 重み付き距離
        (dl * dl + dc * dc + h_diff * h_diff).sqrt()
    }

    /// 指定した許容範囲内かどうか判定
    pub fn is_within_tolerance(&self, target: &Lch, tolerance: f32) -> bool {
        let normalized_tolerance = tolerance * 200.0;
        self.distance(target) < normalized_tolerance
    }
}

/// YUV色構造体
#[derive(Debug, Clone, Copy)]
pub struct Yuv {
    /// Y: 輝度 (0.0 - 1.0)
    pub y: f32,
    /// U: 青-黄軸 (-0.5 - 0.5)
    pub u: f32,
    /// V: 赤-緑軸 (-0.5 - 0.5)
    pub v: f32,
}

impl Yuv {
    pub fn new(y: f32, u: f32, v: f32) -> Self {
        Self { y, u, v }
    }

    /// 2つのYUV色の距離を計算
    pub fn distance(&self, other: &Yuv) -> f32 {
        let dy = self.y - other.y;
        let du = self.u - other.u;
        let dv = self.v - other.v;
        (dy * dy + du * du + dv * dv).sqrt()
    }

    /// 指定した許容範囲内かどうか判定
    pub fn is_within_tolerance(&self, target: &Yuv, tolerance: f32) -> bool {
        self.distance(target) < tolerance
    }
}

/// RGB → HSV 変換
pub fn rgb_to_hsv(rgb: &Rgb) -> Hsv {
    let (r, g, b) = rgb.normalized();

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    // 明度 (V)
    let v = max;

    // 彩度 (S)
    let s = if max == 0.0 { 0.0 } else { delta / max };

    // 色相 (H)
    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    // 色相が負の場合は360を足す
    let h = if h < 0.0 { h + 360.0 } else { h };

    Hsv::new(h, s, v)
}

/// HSV → RGB 変換
pub fn hsv_to_rgb(hsv: &Hsv) -> Rgb {
    let h = hsv.h;
    let s = hsv.s;
    let v = hsv.v;

    if s == 0.0 {
        // 無彩色
        let val = (v * 255.0) as u8;
        return Rgb::new(val, val, val);
    }

    let h = h / 60.0;
    let i = h.floor() as i32;
    let f = h - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    let (r, g, b) = match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        5 => (v, p, q),
        _ => (v, t, p),
    };

    Rgb::new((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

/// RGB → LAB 変換
/// 
/// RGB → XYZ → LAB の順で変換
pub fn rgb_to_lab(rgb: &Rgb) -> Lab {
    let (r, g, b) = rgb.normalized();
    
    // RGB → XYZ 変換（sRGB色空間、D65白色点）
    // ガンマ補正
    let r_linear = if r <= 0.04045 {
        r / 12.92
    } else {
        ((r + 0.055) / 1.055).powf(2.4)
    };
    let g_linear = if g <= 0.04045 {
        g / 12.92
    } else {
        ((g + 0.055) / 1.055).powf(2.4)
    };
    let b_linear = if b <= 0.04045 {
        b / 12.92
    } else {
        ((b + 0.055) / 1.055).powf(2.4)
    };
    
    // sRGB to XYZ (D65)
    let x = r_linear * 0.4124564 + g_linear * 0.3575761 + b_linear * 0.1804375;
    let y = r_linear * 0.2126729 + g_linear * 0.7151522 + b_linear * 0.0721750;
    let z = r_linear * 0.0193339 + g_linear * 0.1191920 + b_linear * 0.9503041;
    
    // XYZ → LAB 変換（D65白色点: Xn=0.95047, Yn=1.0, Zn=1.08883）
    let xn = 0.95047;
    let yn = 1.0;
    let zn = 1.08883;
    
    let fx = lab_f(x / xn);
    let fy = lab_f(y / yn);
    let fz = lab_f(z / zn);
    
    let l = 116.0 * fy - 16.0;
    let a = 500.0 * (fx - fy);
    let b = 200.0 * (fy - fz);
    
    Lab::new(l, a, b)
}

/// LAB変換用の補助関数
#[inline]
fn lab_f(t: f32) -> f32 {
    const DELTA: f32 = 6.0 / 29.0;
    const DELTA_CUBED: f32 = DELTA * DELTA * DELTA;
    const THREE_DELTA_SQUARED: f32 = 3.0 * DELTA * DELTA;
    
    if t > DELTA_CUBED {
        t.cbrt()
    } else {
        t / THREE_DELTA_SQUARED + 4.0 / 29.0
    }
}

/// RGB → LCH 変換
/// 
/// RGB → LAB → LCH の順で変換
pub fn rgb_to_lch(rgb: &Rgb) -> Lch {
    let lab = rgb_to_lab(rgb);
    
    let l = lab.l;
    let c = (lab.a * lab.a + lab.b * lab.b).sqrt();
    let h = lab.b.atan2(lab.a).to_degrees();
    let h = if h < 0.0 { h + 360.0 } else { h };
    
    Lch::new(l, c, h)
}

/// RGB → YUV 変換
/// 
/// ITU-R BT.601標準を使用
pub fn rgb_to_yuv(rgb: &Rgb) -> Yuv {
    let (r, g, b) = rgb.normalized();
    
    let y = 0.299 * r + 0.587 * g + 0.114 * b;
    let u = -0.14713 * r - 0.28886 * g + 0.436 * b;
    let v = 0.615 * r - 0.51499 * g - 0.10001 * b;
    
    Yuv::new(y, u, v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_hsv_green() {
        let rgb = Rgb::new(0, 255, 0);
        let hsv = rgb_to_hsv(&rgb);
        assert!((hsv.h - 120.0).abs() < 0.1);
        assert!((hsv.s - 1.0).abs() < 0.01);
        assert!((hsv.v - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_rgb_to_hsv_red() {
        let rgb = Rgb::new(255, 0, 0);
        let hsv = rgb_to_hsv(&rgb);
        assert!(hsv.h < 1.0 || hsv.h > 359.0); // 0度付近
        assert!((hsv.s - 1.0).abs() < 0.01);
        assert!((hsv.v - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_rgb_to_hsv_blue() {
        let rgb = Rgb::new(0, 0, 255);
        let hsv = rgb_to_hsv(&rgb);
        assert!((hsv.h - 240.0).abs() < 0.1);
        assert!((hsv.s - 1.0).abs() < 0.01);
        assert!((hsv.v - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_hsv_to_rgb_green() {
        let hsv = Hsv::new(120.0, 1.0, 1.0);
        let rgb = hsv_to_rgb(&hsv);
        assert_eq!(rgb.r, 0);
        assert_eq!(rgb.g, 255);
        assert_eq!(rgb.b, 0);
    }

    #[test]
    fn test_roundtrip() {
        let original = Rgb::new(128, 64, 192);
        let hsv = rgb_to_hsv(&original);
        let converted = hsv_to_rgb(&hsv);
        // 変換誤差を許容
        assert!((original.r as i16 - converted.r as i16).abs() <= 1);
        assert!((original.g as i16 - converted.g as i16).abs() <= 1);
        assert!((original.b as i16 - converted.b as i16).abs() <= 1);
    }

    #[test]
    fn test_rgb_to_lab_green() {
        let rgb = Rgb::new(0, 255, 0);
        let lab = rgb_to_lab(&rgb);
        // 緑はa*が負の値になるはず
        assert!(lab.a < 0.0);
        assert!(lab.l > 50.0); // 明るい色
    }

    #[test]
    fn test_rgb_to_lch_green() {
        let rgb = Rgb::new(0, 255, 0);
        let lch = rgb_to_lch(&rgb);
        // 緑の色相は約120度
        assert!((lch.h - 120.0).abs() < 10.0);
        assert!(lch.c > 50.0); // 彩度が高い
    }

    #[test]
    fn test_rgb_to_yuv() {
        let rgb = Rgb::new(128, 128, 128);
        let yuv = rgb_to_yuv(&rgb);
        // グレーはUとVが0に近い
        assert!(yuv.u.abs() < 0.1);
        assert!(yuv.v.abs() < 0.1);
    }

    #[test]
    fn test_lab_distance() {
        let lab1 = Lab::new(50.0, 0.0, 0.0);
        let lab2 = Lab::new(50.0, 10.0, 10.0);
        let dist = lab1.distance(&lab2);
        assert!(dist > 0.0);
        assert!(dist < 20.0);
    }

    #[test]
    fn test_lch_distance() {
        let lch1 = Lch::new(50.0, 50.0, 120.0);
        let lch2 = Lch::new(50.0, 50.0, 130.0);
        let dist = lch1.distance(&lch2);
        assert!(dist > 0.0);
        assert!(dist < 20.0);
    }
}

