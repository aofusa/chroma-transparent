//! モルフォロジー演算

use image::GrayImage;

/// 3x3カーネルでの収縮処理
/// ノイズ除去に使用
pub fn erode(mask: &GrayImage, iterations: u32) -> GrayImage {
    let mut result = mask.clone();

    for _ in 0..iterations {
        result = erode_once(&result);
    }

    result
}

/// 3x3カーネルでの膨張処理
/// エッジ拡張に使用
pub fn dilate(mask: &GrayImage, iterations: u32) -> GrayImage {
    let mut result = mask.clone();

    for _ in 0..iterations {
        result = dilate_once(&result);
    }

    result
}

/// 1回の収縮処理
fn erode_once(mask: &GrayImage) -> GrayImage {
    let (width, height) = mask.dimensions();
    let mut result = GrayImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            // 3x3カーネル内の最小値を取得
            let mut min_val = 255u8;

            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;

                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let val = mask.get_pixel(nx as u32, ny as u32).0[0];
                        min_val = min_val.min(val);
                    }
                }
            }

            result.put_pixel(x, y, image::Luma([min_val]));
        }
    }

    result
}

/// 1回の膨張処理
fn dilate_once(mask: &GrayImage) -> GrayImage {
    let (width, height) = mask.dimensions();
    let mut result = GrayImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            // 3x3カーネル内の最大値を取得
            let mut max_val = 0u8;

            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;

                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let val = mask.get_pixel(nx as u32, ny as u32).0[0];
                        max_val = max_val.max(val);
                    }
                }
            }

            result.put_pixel(x, y, image::Luma([max_val]));
        }
    }

    result
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
}

