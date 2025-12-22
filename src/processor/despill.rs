//! デスピル処理（色かぶり除去）

use image::RgbaImage;

use crate::color::Rgb;

/// クロマキー色の「かぶり」を除去
/// 被写体の縁に残った背景色の反射を軽減
///
/// # Arguments
/// * `image` - 処理対象画像（インプレース変更）
/// * `target_color` - クロマキー対象色
/// * `strength` - デスピル強度 (0.0 - 1.0)
pub fn despill(image: &mut RgbaImage, target_color: &Rgb, strength: f32) {
    if strength <= 0.0 {
        return;
    }

    let strength = strength.min(1.0);

    // どのチャンネルがメインのクロマキー色かを判定
    let dominant_channel = get_dominant_channel(target_color);

    for pixel in image.pixels_mut() {
        let r = pixel.0[0] as i16;
        let g = pixel.0[1] as i16;
        let b = pixel.0[2] as i16;

        // ドミナントチャンネルの過剰量を計算
        let excess = match dominant_channel {
            ColorChannel::Red => {
                let max_gb = g.max(b);
                (r - max_gb).max(0)
            }
            ColorChannel::Green => {
                let max_rb = r.max(b);
                (g - max_rb).max(0)
            }
            ColorChannel::Blue => {
                let max_rg = r.max(g);
                (b - max_rg).max(0)
            }
        };

        // 閾値（小さすぎる差は無視）
        if excess <= 20 {
            continue;
        }

        // 過剰量を削減
        let reduction = (excess as f32 * strength) as i16;

        match dominant_channel {
            ColorChannel::Red => {
                pixel.0[0] = (r - reduction).max(0) as u8;
            }
            ColorChannel::Green => {
                pixel.0[1] = (g - reduction).max(0) as u8;
            }
            ColorChannel::Blue => {
                pixel.0[2] = (b - reduction).max(0) as u8;
            }
        }
    }
}

/// 色チャンネル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColorChannel {
    Red,
    Green,
    Blue,
}

/// ドミナント（支配的）なカラーチャンネルを取得
fn get_dominant_channel(color: &Rgb) -> ColorChannel {
    if color.r >= color.g && color.r >= color.b {
        ColorChannel::Red
    } else if color.g >= color.r && color.g >= color.b {
        ColorChannel::Green
    } else {
        ColorChannel::Blue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_despill_green() {
        let mut image = RgbaImage::new(1, 1);
        // 緑かぶりのあるピクセル（緑が他より高い）
        image.put_pixel(0, 0, Rgba([100, 200, 80, 255]));

        let target = Rgb::new(0, 255, 0); // 緑
        despill(&mut image, &target, 1.0);

        let pixel = image.get_pixel(0, 0);
        // 緑が削減されている
        assert!(pixel.0[1] < 200);
        // 赤と青は変わらない
        assert_eq!(pixel.0[0], 100);
        assert_eq!(pixel.0[2], 80);
    }

    #[test]
    fn test_despill_blue() {
        let mut image = RgbaImage::new(1, 1);
        // 青かぶりのあるピクセル
        image.put_pixel(0, 0, Rgba([100, 100, 200, 255]));

        let target = Rgb::new(0, 0, 255); // 青
        despill(&mut image, &target, 1.0);

        let pixel = image.get_pixel(0, 0);
        // 青が削減されている
        assert!(pixel.0[2] < 200);
    }

    #[test]
    fn test_despill_zero_strength() {
        let mut image = RgbaImage::new(1, 1);
        image.put_pixel(0, 0, Rgba([100, 200, 80, 255]));

        let target = Rgb::new(0, 255, 0);
        despill(&mut image, &target, 0.0);

        let pixel = image.get_pixel(0, 0);
        // 変化なし
        assert_eq!(pixel.0[1], 200);
    }

    #[test]
    fn test_get_dominant_channel() {
        assert_eq!(
            get_dominant_channel(&Rgb::new(255, 0, 0)),
            ColorChannel::Red
        );
        assert_eq!(
            get_dominant_channel(&Rgb::new(0, 255, 0)),
            ColorChannel::Green
        );
        assert_eq!(
            get_dominant_channel(&Rgb::new(0, 0, 255)),
            ColorChannel::Blue
        );
    }
}

