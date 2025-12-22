//! SIMD最適化モジュール（改善案E）
//!
//! プラットフォーム別のSIMD実装とフォールバック処理
//!
//! 対応アーキテクチャ:
//! - x86_64: SSE2/AVX2
//! - aarch64: NEON
//! - その他: スカラーフォールバック

/// アルファチャンネルの一括適用（SIMD最適化）
///
/// RGBAバッファの各ピクセルのアルファ値を、別のアルファバッファから設定
pub fn apply_alpha_simd(rgba: &mut [u8], alpha: &[u8]) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe { apply_alpha_avx2(rgba, alpha) };
            return;
        }
        if is_x86_feature_detected!("sse2") {
            unsafe { apply_alpha_sse2(rgba, alpha) };
            return;
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        unsafe { apply_alpha_neon(rgba, alpha) };
        return;
    }

    // フォールバック
    #[allow(unreachable_code)]
    apply_alpha_scalar(rgba, alpha);
}

/// マスクの反転（SIMD最適化）
///
/// 255 - value の計算を高速化
pub fn invert_mask_simd(src: &[u8], dst: &mut [u8]) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe { invert_mask_avx2(src, dst) };
            return;
        }
        if is_x86_feature_detected!("sse2") {
            unsafe { invert_mask_sse2(src, dst) };
            return;
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        unsafe { invert_mask_neon(src, dst) };
        return;
    }

    // フォールバック
    #[allow(unreachable_code)]
    invert_mask_scalar(src, dst);
}

// === x86_64 SSE2 実装 ===

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn apply_alpha_sse2(rgba: &mut [u8], alpha: &[u8]) {
    let pixel_count = alpha.len();
    let mut i = 0;

    // 16バイト（4ピクセル）ずつ処理
    while i + 4 <= pixel_count {
        let rgba_ptr = rgba.as_mut_ptr().add(i * 4);

        // 4ピクセル分のアルファ値を読み込み
        let a0 = *alpha.get_unchecked(i);
        let a1 = *alpha.get_unchecked(i + 1);
        let a2 = *alpha.get_unchecked(i + 2);
        let a3 = *alpha.get_unchecked(i + 3);

        // 各ピクセルのアルファチャンネル位置に直接書き込み
        *rgba_ptr.add(3) = a0;
        *rgba_ptr.add(7) = a1;
        *rgba_ptr.add(11) = a2;
        *rgba_ptr.add(15) = a3;

        i += 4;
    }

    // 残りはスカラー処理
    while i < pixel_count {
        rgba[i * 4 + 3] = alpha[i];
        i += 1;
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn invert_mask_sse2(src: &[u8], dst: &mut [u8]) {
    use std::arch::x86_64::*;

    let len = src.len().min(dst.len());
    let mut i = 0;

    // 255を全バイトに持つベクトル
    let all_255 = _mm_set1_epi8(-1i8); // 0xFF

    // 16バイトずつ処理
    while i + 16 <= len {
        let src_vec = _mm_loadu_si128(src.as_ptr().add(i) as *const __m128i);
        let result = _mm_sub_epi8(all_255, src_vec);
        _mm_storeu_si128(dst.as_mut_ptr().add(i) as *mut __m128i, result);
        i += 16;
    }

    // 残りはスカラー処理
    while i < len {
        *dst.get_unchecked_mut(i) = 255 - *src.get_unchecked(i);
        i += 1;
    }
}

// === x86_64 AVX2 実装 ===

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn apply_alpha_avx2(rgba: &mut [u8], alpha: &[u8]) {
    let pixel_count = alpha.len();
    let mut i = 0;

    // 8ピクセルずつ処理
    while i + 8 <= pixel_count {
        let rgba_ptr = rgba.as_mut_ptr().add(i * 4);

        // 8ピクセル分のアルファ値を読み込み、各ピクセルのアルファ位置に書き込み
        for j in 0..8 {
            *rgba_ptr.add(j * 4 + 3) = *alpha.get_unchecked(i + j);
        }

        i += 8;
    }

    // 残りはスカラー処理
    while i < pixel_count {
        rgba[i * 4 + 3] = alpha[i];
        i += 1;
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn invert_mask_avx2(src: &[u8], dst: &mut [u8]) {
    use std::arch::x86_64::*;

    let len = src.len().min(dst.len());
    let mut i = 0;

    // 255を全バイトに持つベクトル
    let all_255 = _mm256_set1_epi8(-1i8); // 0xFF

    // 32バイトずつ処理
    while i + 32 <= len {
        let src_vec = _mm256_loadu_si256(src.as_ptr().add(i) as *const __m256i);
        let result = _mm256_sub_epi8(all_255, src_vec);
        _mm256_storeu_si256(dst.as_mut_ptr().add(i) as *mut __m256i, result);
        i += 32;
    }

    // 残りはSSE2で処理
    let all_255_sse = _mm_set1_epi8(-1i8);
    while i + 16 <= len {
        let src_vec = _mm_loadu_si128(src.as_ptr().add(i) as *const __m128i);
        let result = _mm_sub_epi8(all_255_sse, src_vec);
        _mm_storeu_si128(dst.as_mut_ptr().add(i) as *mut __m128i, result);
        i += 16;
    }

    // 残りはスカラー処理
    while i < len {
        *dst.get_unchecked_mut(i) = 255 - *src.get_unchecked(i);
        i += 1;
    }
}

// === aarch64 NEON 実装 ===

#[cfg(target_arch = "aarch64")]
unsafe fn apply_alpha_neon(rgba: &mut [u8], alpha: &[u8]) {
    use std::arch::aarch64::*;

    let pixel_count = alpha.len();
    let mut i = 0;

    // 16ピクセルずつ処理
    while i + 16 <= pixel_count {
        let rgba_ptr = rgba.as_mut_ptr().add(i * 4);

        // 16ピクセル分のアルファ値を読み込み
        let alpha_vec = vld1q_u8(alpha.as_ptr().add(i));

        // 各ピクセルのアルファチャンネル位置に書き込み
        for j in 0..16 {
            *rgba_ptr.add(j * 4 + 3) = vgetq_lane_u8(alpha_vec, j as i32);
        }

        i += 16;
    }

    // 残りはスカラー処理
    while i < pixel_count {
        rgba[i * 4 + 3] = alpha[i];
        i += 1;
    }
}

#[cfg(target_arch = "aarch64")]
unsafe fn invert_mask_neon(src: &[u8], dst: &mut [u8]) {
    use std::arch::aarch64::*;

    let len = src.len().min(dst.len());
    let mut i = 0;

    // 255を全バイトに持つベクトル
    let all_255 = vdupq_n_u8(255);

    // 16バイトずつ処理
    while i + 16 <= len {
        let src_vec = vld1q_u8(src.as_ptr().add(i));
        let result = vsubq_u8(all_255, src_vec);
        vst1q_u8(dst.as_mut_ptr().add(i), result);
        i += 16;
    }

    // 残りはスカラー処理
    while i < len {
        *dst.get_unchecked_mut(i) = 255 - *src.get_unchecked(i);
        i += 1;
    }
}

// === スカラーフォールバック ===

/// スカラーフォールバック: アルファ適用
fn apply_alpha_scalar(rgba: &mut [u8], alpha: &[u8]) {
    for (i, &a) in alpha.iter().enumerate() {
        if i * 4 + 3 < rgba.len() {
            rgba[i * 4 + 3] = a;
        }
    }
}

/// スカラーフォールバック: マスク反転
fn invert_mask_scalar(src: &[u8], dst: &mut [u8]) {
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *d = 255 - s;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_alpha_simd() {
        let mut rgba = vec![255u8, 0, 0, 0, 0, 255, 0, 0, 0, 0, 255, 0];
        let alpha = vec![100u8, 150, 200];

        apply_alpha_simd(&mut rgba, &alpha);

        assert_eq!(rgba[3], 100);
        assert_eq!(rgba[7], 150);
        assert_eq!(rgba[11], 200);
    }

    #[test]
    fn test_invert_mask_simd() {
        let src = vec![0u8, 255, 128, 64];
        let mut dst = vec![0u8; 4];

        invert_mask_simd(&src, &mut dst);

        assert_eq!(dst[0], 255);
        assert_eq!(dst[1], 0);
        assert_eq!(dst[2], 127);
        assert_eq!(dst[3], 191);
    }

    #[test]
    fn test_large_buffer() {
        // 大きなバッファでテスト
        let mut rgba = vec![0u8; 4096 * 4];
        let alpha: Vec<u8> = (0..4096).map(|i| (i % 256) as u8).collect();

        apply_alpha_simd(&mut rgba, &alpha);

        for i in 0..4096 {
            assert_eq!(rgba[i * 4 + 3], (i % 256) as u8);
        }
    }
}

