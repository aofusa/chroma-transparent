//! 色空間変換

use super::{Hsv, Rgb};

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
}

