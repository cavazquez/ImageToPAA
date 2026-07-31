//! Integration tests for the encode pipeline and fixtures.

use image::RgbaImage;
use image_to_paa::{EncodeOptions, PaFormat, encode_rgba8, parse_paa};
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/sources")
}

fn load(name: &str) -> RgbaImage {
    let path = fixtures_dir().join(name);
    image::open(&path)
        .unwrap_or_else(|e| panic!("open {}: {e}", path.display()))
        .to_rgba8()
}

#[test]
fn fixture_opaque_dxt1() {
    let img = load("opaque_blocks_16.png");
    let bytes = encode_rgba8(&img, EncodeOptions::default()).unwrap();
    assert_eq!(&bytes[0..2], [0x01, 0xFF]);
    let p = parse_paa(&bytes).unwrap();
    assert_eq!(p.format, PaFormat::Dxt1);
    assert!(p.mips.len() >= 2);
    assert!(!p.tags.iter().any(|(n, _)| n == "GALF"));
}

#[test]
fn fixture_soft_dxt5() {
    let img = load("soft_radial_16.png");
    let bytes = encode_rgba8(&img, EncodeOptions::default()).unwrap();
    assert_eq!(&bytes[0..2], [0x05, 0xFF]);
    let p = parse_paa(&bytes).unwrap();
    assert_eq!(p.format, PaFormat::Dxt5);
    assert!(
        p.tags
            .iter()
            .any(|(n, v)| n == "GALF" && v.as_slice() == [1, 0, 0, 0])
    );
}

#[test]
fn fixture_no_mips_rectangular() {
    let img = load("soft_diagonal_8x16.png");
    let bytes = encode_rgba8(
        &img,
        EncodeOptions {
            format: PaFormat::Auto,
            generate_mips: false,
            compress: false,
        },
    )
    .unwrap();
    let p = parse_paa(&bytes).unwrap();
    assert_eq!(p.mips.len(), 1);
    assert_eq!((p.mips[0].width, p.mips[0].height), (8, 16));
}
