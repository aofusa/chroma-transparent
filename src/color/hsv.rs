//! HSV色構造体

/// HSV色構造体
#[derive(Debug, Clone, Copy)]
pub struct Hsv {
    /// 色相: 0.0 - 360.0
    pub h: f32,
    /// 彩度: 0.0 - 1.0
    pub s: f32,
    /// 明度: 0.0 - 1.0
    pub v: f32,
}

impl Hsv {
    /// 新しいHSV色を作成
    pub fn new(h: f32, s: f32, v: f32) -> Self {
        Self { h, s, v }
    }

    /// 2つのHSV色の距離を計算
    /// 色相は円環として計算（0度と360度が隣接）
    /// 返り値は0.0〜1.0に正規化された距離
    pub fn distance(&self, other: &Hsv) -> f32 {
        // 色相の差（円環距離）
        let h_diff = {
            let diff = (self.h - other.h).abs();
            let diff = if diff > 180.0 { 360.0 - diff } else { diff };
            diff / 180.0 // 0.0 - 1.0 に正規化
        };

        // 彩度の差
        let s_diff = (self.s - other.s).abs();

        // 明度の差
        let v_diff = (self.v - other.v).abs();

        // 重み付き距離（色相を重視）
        // 色相が近く、彩度と明度も近ければ距離は小さい
        let distance = (h_diff * 0.5 + s_diff * 0.3 + v_diff * 0.2).sqrt();
        distance.min(1.0)
    }

    /// 指定した許容範囲内かどうか判定
    pub fn is_within_tolerance(&self, target: &Hsv, tolerance: f32) -> bool {
        self.distance(target) < tolerance
    }
}

impl Default for Hsv {
    fn default() -> Self {
        Self::new(120.0, 1.0, 1.0) // 緑
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_color_distance() {
        let hsv1 = Hsv::new(120.0, 1.0, 1.0);
        let hsv2 = Hsv::new(120.0, 1.0, 1.0);
        assert!(hsv1.distance(&hsv2) < 0.001);
    }

    #[test]
    fn test_opposite_hue_distance() {
        let hsv1 = Hsv::new(0.0, 1.0, 1.0);
        let hsv2 = Hsv::new(180.0, 1.0, 1.0);
        let dist = hsv1.distance(&hsv2);
        assert!(dist > 0.5);
    }

    #[test]
    fn test_circular_hue() {
        let hsv1 = Hsv::new(10.0, 1.0, 1.0);
        let hsv2 = Hsv::new(350.0, 1.0, 1.0);
        let dist = hsv1.distance(&hsv2);
        // 20度の差なので距離は小さいはず
        assert!(dist < 0.3);
    }

    #[test]
    fn test_within_tolerance() {
        let target = Hsv::new(120.0, 1.0, 1.0);
        let similar = Hsv::new(125.0, 0.95, 0.95);
        assert!(similar.is_within_tolerance(&target, 0.3));
    }
}

