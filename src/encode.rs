//! Full encode pipeline: mips → BCn → tags → container.

use crate::codec::{encode_bc1, encode_bc3};
use crate::container::{write_paa, MipPayload};
use crate::error::EncodeError;
use crate::metadata::{build_semantic_tags, colour_stats};
use crate::mipmaps::generate_mip_chain;
use crate::options::{EncodeOptions, PaFormat};
use image::RgbaImage;
use std::io::Write;

/// Encode an RGBA8 image into a PAA byte stream (no filesystem I/O).
pub fn encode_rgba8(image: &RgbaImage, options: EncodeOptions) -> Result<Vec<u8>, EncodeError> {
    let mut buf = Vec::new();
    encode_rgba8_to(&mut buf, image, options)?;
    Ok(buf)
}

/// Encode into any [`Write`] destination.
pub fn encode_rgba8_to<W: Write>(
    w: W,
    image: &RgbaImage,
    options: EncodeOptions,
) -> Result<(), EncodeError> {
    let (width, height) = image.dimensions();
    validate_dimensions(width, height)?;

    let stats = colour_stats(image);
    let format = resolve_format(options.format, stats.has_alpha)?;

    let chain = generate_mip_chain(image, options.generate_mips);
    if chain.len() > 16 {
        return Err(EncodeError::TooManyMips { count: chain.len() });
    }

    let mut mips = Vec::with_capacity(chain.len());
    for level in &chain {
        let (w, h) = level.dimensions();
        let raw = level.as_raw();
        let data = match format {
            PaFormat::Dxt1 => encode_bc1(raw, w, h)?,
            PaFormat::Dxt5 => encode_bc3(raw, w, h)?,
            PaFormat::Auto => unreachable!(),
        };
        mips.push(MipPayload {
            width: w as u16,
            height: h as u16,
            data,
        });
    }

    let tags = build_semantic_tags(image, format);
    write_paa(w, format, &tags, &mips)
}

fn resolve_format(requested: PaFormat, has_alpha: bool) -> Result<PaFormat, EncodeError> {
    match requested {
        PaFormat::Auto => Ok(if has_alpha {
            PaFormat::Dxt5
        } else {
            PaFormat::Dxt1
        }),
        PaFormat::Dxt5 => Ok(PaFormat::Dxt5),
        PaFormat::Dxt1 => {
            if has_alpha {
                Err(EncodeError::SoftAlphaForcedDxt1)
            } else {
                Ok(PaFormat::Dxt1)
            }
        }
    }
}

fn validate_dimensions(width: u32, height: u32) -> Result<(), EncodeError> {
    if width == 0 || height == 0 {
        return Err(EncodeError::EmptyImage);
    }
    if !width.is_power_of_two() || !height.is_power_of_two() {
        return Err(EncodeError::NotPowerOfTwo { width, height });
    }
    if width < 4 || height < 4 {
        return Err(EncodeError::TooSmall { width, height });
    }
    if width > 4096 || height > 4096 {
        return Err(EncodeError::TooLarge { width, height });
    }
    if width % 4 != 0 || height % 4 != 0 {
        return Err(EncodeError::NotBlockAligned { width, height });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{decompress, BcFormat};
    use crate::container::parse_paa;
    use image::Rgba;

    #[test]
    fn rejects_non_pot() {
        let img = RgbaImage::from_pixel(3, 4, Rgba([0, 0, 0, 0]));
        assert!(matches!(
            encode_rgba8(&img, EncodeOptions::default()),
            Err(EncodeError::NotPowerOfTwo { .. })
        ));
    }

    #[test]
    fn no_stub_magic() {
        let img = RgbaImage::from_pixel(4, 4, Rgba([10, 20, 30, 40]));
        let bytes = encode_rgba8(&img, EncodeOptions::default()).unwrap();
        assert_eq!(&bytes[0..2], [0x05, 0xFF]);
        assert!(!bytes.windows(4).any(|w| w == b"PAA\0"));
        assert!(!bytes.windows(13).any(|w| w == b"TODO_REAL_PAA"));
    }

    #[test]
    fn opaque_dxt1_with_mips() {
        let img = RgbaImage::from_pixel(16, 16, Rgba([40, 80, 120, 255]));
        let bytes = encode_rgba8(&img, EncodeOptions::default()).unwrap();
        let p = parse_paa(&bytes).unwrap();
        assert_eq!(p.format, PaFormat::Dxt1);
        assert_eq!(p.mips.len(), 3); // 16, 8, 4
        let dec = decompress(BcFormat::Bc1, &p.mips[0].data, 16, 16);
        assert_eq!(dec.len(), 16 * 16 * 4);
    }

    #[test]
    fn soft_alpha_dxt5_no_mips() {
        let img = RgbaImage::from_pixel(8, 8, Rgba([200, 10, 10, 128]));
        let bytes = encode_rgba8(
            &img,
            EncodeOptions {
                format: PaFormat::Auto,
                generate_mips: false,
            },
        )
        .unwrap();
        let p = parse_paa(&bytes).unwrap();
        assert_eq!(p.format, PaFormat::Dxt5);
        assert_eq!(p.mips.len(), 1);
        assert!(p
            .tags
            .iter()
            .any(|(n, v)| n == "GALF" && v == &[1, 0, 0, 0]));
        let dec = decompress(BcFormat::Bc3, &p.mips[0].data, 8, 8);
        let mid = dec
            .chunks(4)
            .filter(|px| (100..200).contains(&px[3]))
            .count();
        assert!(mid > 0 || dec.chunks(4).any(|px| px[3] > 0 && px[3] < 255));
    }

    #[test]
    fn rectangular_chain() {
        let img = RgbaImage::from_pixel(8, 16, Rgba([0, 255, 0, 200]));
        let bytes = encode_rgba8(&img, EncodeOptions::default()).unwrap();
        let p = parse_paa(&bytes).unwrap();
        let dims: Vec<_> = p.mips.iter().map(|m| (m.width, m.height)).collect();
        assert_eq!(dims, vec![(8, 16), (4, 8)]);
    }

    #[test]
    fn forced_dxt1_with_alpha_errors() {
        let img = RgbaImage::from_pixel(4, 4, Rgba([0, 0, 0, 128]));
        let err = encode_rgba8(
            &img,
            EncodeOptions {
                format: PaFormat::Dxt1,
                generate_mips: false,
            },
        )
        .unwrap_err();
        assert_eq!(err, EncodeError::SoftAlphaForcedDxt1);
    }
}
