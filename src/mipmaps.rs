//! Alpha-correct, sRGB-aware mipmap chain (independent of BCn / container).

use image::{Rgba, RgbaImage};

/// Generate mip levels from `base` without mutating it.
/// Stops after including the level where `min(w,h) == 4`.
/// If `generate` is false, returns only the base clone.
pub fn generate_mip_chain(base: &RgbaImage, generate: bool) -> Vec<RgbaImage> {
    let mut chain = vec![base.clone()];
    if !generate {
        return chain;
    }

    let mut current = base.clone();
    loop {
        let (w, h) = current.dimensions();
        if w.min(h) <= 4 {
            break;
        }
        let next = downsample_alpha_correct(&current);
        let (nw, nh) = next.dimensions();
        chain.push(next.clone());
        if nw.min(nh) <= 4 {
            break;
        }
        current = next;
    }
    chain
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

fn downsample_alpha_correct(src: &RgbaImage) -> RgbaImage {
    let (w, h) = src.dimensions();
    let nw = (w / 2).max(1);
    let nh = (h / 2).max(1);
    let mut out = RgbaImage::new(nw, nh);

    for y in 0..nh {
        for x in 0..nw {
            let x0 = x * 2;
            let y0 = y * 2;
            let samples = [
                src.get_pixel(x0.min(w - 1), y0.min(h - 1)).0,
                src.get_pixel((x0 + 1).min(w - 1), y0.min(h - 1)).0,
                src.get_pixel(x0.min(w - 1), (y0 + 1).min(h - 1)).0,
                src.get_pixel((x0 + 1).min(w - 1), (y0 + 1).min(h - 1)).0,
            ];

            let mut sum_r = 0.0f32;
            let mut sum_g = 0.0f32;
            let mut sum_b = 0.0f32;
            let mut sum_a = 0.0f32;

            for px in &samples {
                let a = px[3] as f32 / 255.0;
                let r = srgb_to_linear(px[0] as f32 / 255.0) * a;
                let g = srgb_to_linear(px[1] as f32 / 255.0) * a;
                let b = srgb_to_linear(px[2] as f32 / 255.0) * a;
                sum_r += r;
                sum_g += g;
                sum_b += b;
                sum_a += a;
            }

            let inv = 1.0 / 4.0;
            sum_r *= inv;
            sum_g *= inv;
            sum_b *= inv;
            sum_a *= inv;

            let (r, g, b) = if sum_a > 1e-6 {
                (
                    linear_to_srgb(sum_r / sum_a),
                    linear_to_srgb(sum_g / sum_a),
                    linear_to_srgb(sum_b / sum_a),
                )
            } else {
                (0.0, 0.0, 0.0)
            };

            out.put_pixel(
                x,
                y,
                Rgba([
                    (r * 255.0).round().clamp(0.0, 255.0) as u8,
                    (g * 255.0).round().clamp(0.0, 255.0) as u8,
                    (b * 255.0).round().clamp(0.0, 255.0) as u8,
                    (sum_a * 255.0).round().clamp(0.0, 255.0) as u8,
                ]),
            );
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn chain_1024x2048_nine_levels() {
        let img = RgbaImage::from_pixel(1024, 2048, Rgba([0, 0, 0, 0]));
        let chain = generate_mip_chain(&img, true);
        let dims: Vec<_> = chain.iter().map(|i| i.dimensions()).collect();
        assert_eq!(
            dims,
            vec![
                (1024, 2048),
                (512, 1024),
                (256, 512),
                (128, 256),
                (64, 128),
                (32, 64),
                (16, 32),
                (8, 16),
                (4, 8),
            ]
        );
    }

    #[test]
    fn chain_512_eight_levels() {
        let img = RgbaImage::from_pixel(512, 512, Rgba([1, 2, 3, 255]));
        let chain = generate_mip_chain(&img, true);
        assert_eq!(chain.len(), 8);
        assert_eq!(chain.last().unwrap().dimensions(), (4, 4));
        assert_eq!(chain[0].get_pixel(0, 0).0, [1, 2, 3, 255]);
    }

    #[test]
    fn no_mips_single_level() {
        let img = RgbaImage::from_pixel(64, 32, Rgba([9, 9, 9, 9]));
        let chain = generate_mip_chain(&img, false);
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].dimensions(), (64, 32));
    }

    #[test]
    fn no_black_halo_over_white() {
        // RGB black under transparent pixels should not bleed when downsampled.
        let mut img = RgbaImage::from_pixel(8, 8, Rgba([0, 0, 0, 0]));
        for y in 2..6 {
            for x in 2..6 {
                img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }
        let chain = generate_mip_chain(&img, true);
        let mip = &chain[1]; // 4x4
                             // Compose over white: transparent pixels must stay white.
        for px in mip.pixels() {
            let a = px.0[3] as f32 / 255.0;
            let r = px.0[0] as f32 * a + 255.0 * (1.0 - a);
            if px.0[3] < 10 {
                assert!(r > 250.0, "halo: composed R={r} for {px:?}");
            }
        }
    }

    #[test]
    fn portrait_and_landscape() {
        let p = generate_mip_chain(&RgbaImage::from_pixel(16, 32, Rgba([0; 4])), true);
        assert_eq!(p.last().unwrap().dimensions(), (4, 8));
        let l = generate_mip_chain(&RgbaImage::from_pixel(32, 16, Rgba([0; 4])), true);
        assert_eq!(l.last().unwrap().dimensions(), (8, 4));
    }
}
