//! Minimal PAA writer for Arma 3 UI / inventory textures.
//!
//! This is an intentional first cut: DXT1/DXT5 with optional mip chain.
//! Format notes live in `docs/paa-format.md`. Validate outputs in TexView2
//! before promoting into a mod PBO.

use anyhow::{bail, Result};
use image::RgbaImage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaFormat {
    Dxt1,
    Dxt5,
}

/// Encode an RGBA8 image into a Bohemia PAA byte stream.
pub fn encode_rgba8(image: &RgbaImage, format: PaFormat, generate_mips: bool) -> Result<Vec<u8>> {
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 {
        bail!("empty image");
    }
    if !width.is_power_of_two() || !height.is_power_of_two() {
        bail!("dimensions must be power-of-two");
    }

    // Placeholder encoder: serializes a documented stub container so the CLI
    // and packing pipeline can land. Real DXT block compression + tag tables
    // land in follow-up commits (see docs/paa-format.md).
    let mut out = Vec::with_capacity(64);
    out.extend_from_slice(b"PAA\0"); // local magic for unfinished payloads
    out.extend_from_slice(&(width as u32).to_le_bytes());
    out.extend_from_slice(&(height as u32).to_le_bytes());
    out.push(match format {
        PaFormat::Dxt1 => 1,
        PaFormat::Dxt5 => 5,
    });
    out.push(u8::from(generate_mips));
    out.extend_from_slice(&(image.len() as u32).to_le_bytes());
    out.extend_from_slice(image.as_raw());

    // Mark clearly unfinished so TexView rejects instead of mis-rendering.
    // Replace with a real BI-compatible stream before RMTFAR promotion.
    out.extend_from_slice(b"TODO_REAL_PAA");
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    fn rejects_non_pot() {
        let img = RgbaImage::from_pixel(3, 4, Rgba([0, 0, 0, 0]));
        assert!(encode_rgba8(&img, PaFormat::Dxt5, true).is_err());
    }

    #[test]
    fn encodes_pot_stub() {
        let img = RgbaImage::from_pixel(4, 4, Rgba([10, 20, 30, 40]));
        let bytes = encode_rgba8(&img, PaFormat::Dxt5, true).unwrap();
        assert!(bytes.starts_with(b"PAA\0"));
        assert!(bytes.ends_with(b"TODO_REAL_PAA"));
    }
}
