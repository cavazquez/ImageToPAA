//! BC1/BC3 compression via `texpresso` 2.0.x (MIT).
//!
//! Backend parameters (deterministic):
//! - `Algorithm::ClusterFit`
//! - `COLOUR_WEIGHTS_PERCEPTUAL`
//! - `weigh_colour_by_alpha = true` for BC3; `false` for BC1
//!
//! Features used: default `texpresso` (no rayon required for determinism).

use crate::error::EncodeError;
use texpresso::{Algorithm, COLOUR_WEIGHTS_PERCEPTUAL, Format, Params};

fn params(weigh_colour_by_alpha: bool) -> Params {
    Params {
        algorithm: Algorithm::ClusterFit,
        weights: COLOUR_WEIGHTS_PERCEPTUAL,
        weigh_colour_by_alpha,
    }
}

/// Encode RGBA8 to BC1. Length = ceil(w/4)*ceil(h/4)*8.
pub fn encode_bc1(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, EncodeError> {
    encode(Format::Bc1, rgba, width, height, false)
}

/// Encode RGBA8 to BC3. Length = ceil(w/4)*ceil(h/4)*16.
pub fn encode_bc3(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, EncodeError> {
    encode(Format::Bc3, rgba, width, height, true)
}

fn encode(
    format: Format,
    rgba: &[u8],
    width: u32,
    height: u32,
    weigh_alpha: bool,
) -> Result<Vec<u8>, EncodeError> {
    let w = width as usize;
    let h = height as usize;
    let expected = w.checked_mul(h).and_then(|n| n.checked_mul(4)).unwrap_or(0);
    if rgba.len() != expected {
        return Err(EncodeError::Write(format!(
            "RGBA buffer length {} != {}x{}x4",
            rgba.len(),
            width,
            height
        )));
    }
    let size = format.compressed_size(w, h);
    let mut out = vec![0u8; size];
    format.compress(rgba, w, h, params(weigh_alpha), &mut out);
    Ok(out)
}

/// Decompress BC1/BC3 for tests (same crate, independent of our wrapper API).
#[cfg(test)]
pub fn decompress(format: Format, data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    format.decompress(data, width as usize, height as usize, &mut out);
    out
}

#[cfg(test)]
pub use texpresso::Format as BcFormat;

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..w * h {
            v.extend_from_slice(&rgba);
        }
        v
    }

    #[test]
    fn bc1_size_4x4_and_rectangular() {
        let a = encode_bc1(&solid(4, 4, [10, 20, 30, 255]), 4, 4).unwrap();
        assert_eq!(a.len(), 8);
        let b = encode_bc1(&solid(8, 4, [10, 20, 30, 255]), 8, 4).unwrap();
        assert_eq!(b.len(), 16);
    }

    #[test]
    fn bc1_deterministic() {
        let src = solid(8, 8, [40, 80, 120, 255]);
        let a = encode_bc1(&src, 8, 8).unwrap();
        let b = encode_bc1(&src, 8, 8).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn bc1_roundtrip_opaque() {
        let src = solid(4, 4, [200, 100, 50, 255]);
        let enc = encode_bc1(&src, 4, 4).unwrap();
        let dec = decompress(Format::Bc1, &enc, 4, 4);
        assert_eq!(dec.len(), 64);
        // Approximate: solid colour should stay opaque.
        for px in dec.chunks(4) {
            assert_eq!(px[3], 255);
        }
    }

    #[test]
    fn bc3_size_and_soft_alpha() {
        let mut src = solid(4, 4, [255, 0, 0, 128]);
        // gradient alpha
        for i in 0..16 {
            src[i * 4 + 3] = (i * 16) as u8;
        }
        let enc = encode_bc3(&src, 4, 4).unwrap();
        assert_eq!(enc.len(), 16);
        let dec = decompress(Format::Bc3, &enc, 4, 4);
        let alphas: Vec<u8> = dec.chunks(4).map(|p| p[3]).collect();
        let unique: std::collections::BTreeSet<_> = alphas.iter().copied().collect();
        assert!(
            unique.len() > 2,
            "soft alpha must keep intermediate levels, got {unique:?}"
        );
    }

    #[test]
    fn bc3_fully_transparent() {
        let src = solid(4, 4, [0, 0, 0, 0]);
        let enc = encode_bc3(&src, 4, 4).unwrap();
        let dec = decompress(Format::Bc3, &enc, 4, 4);
        for px in dec.chunks(4) {
            assert_eq!(px[3], 0);
        }
    }

    #[test]
    fn bc3_psnr_floor_soft_gradient() {
        let mut src = vec![0u8; 8 * 8 * 4];
        for y in 0..8 {
            for x in 0..8 {
                let i = (y * 8 + x) * 4;
                src[i] = 180;
                src[i + 1] = 40;
                src[i + 2] = 90;
                src[i + 3] = ((x + y) * 16).min(255) as u8;
            }
        }
        let enc = encode_bc3(&src, 8, 8).unwrap();
        let dec = decompress(Format::Bc3, &enc, 8, 8);
        let (psnr_rgb, psnr_a, mae_a) = metrics(&src, &dec);
        assert!(psnr_a > 25.0, "alpha PSNR {psnr_a}");
        assert!(mae_a < 20.0, "alpha MAE {mae_a}");
        assert!(psnr_rgb > 20.0, "rgb PSNR {psnr_rgb}");
    }

    fn metrics(a: &[u8], b: &[u8]) -> (f64, f64, f64) {
        let n = a.len() / 4;
        let mut se_rgb = 0.0f64;
        let mut se_a = 0.0f64;
        let mut ae_a = 0.0f64;
        for i in 0..n {
            for c in 0..3 {
                let d = a[i * 4 + c] as f64 - b[i * 4 + c] as f64;
                se_rgb += d * d;
            }
            let d = a[i * 4 + 3] as f64 - b[i * 4 + 3] as f64;
            se_a += d * d;
            ae_a += d.abs();
        }
        let mse_rgb = se_rgb / (n * 3) as f64;
        let mse_a = se_a / n as f64;
        let psnr = |mse: f64| {
            if mse < 1e-12 {
                99.0
            } else {
                10.0 * (255.0f64 * 255.0 / mse).log10()
            }
        };
        (psnr(mse_rgb), psnr(mse_a), ae_a / n as f64)
    }
}
