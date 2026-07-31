//! Structural checks against the Gruppe Adler online-converter oracle.

use image_to_paa::{parse_paa, PaFormat};
use std::fs;
use std::path::PathBuf;

fn oracle_paa() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/oracle/gruppe_adler/r110_independent_icon.paa")
}

#[test]
fn gruppe_adler_r110_icon_structure() {
    let path = oracle_paa();
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    assert_eq!(
        sha256_hex(&bytes),
        "45566b655c6fe2389955764051726308a4bd0085229f7afa89178d757262399b"
    );

    let p = parse_paa(&bytes).expect("parse");
    assert_eq!(p.format, PaFormat::Dxt5);
    let names: Vec<_> = p.tags.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["CGVA", "CXAM", "GALF", "SFFO"]);

    let galf = p.tags.iter().find(|(n, _)| n == "GALF").unwrap();
    assert_eq!(galf.1, [0x01, 0xff, 0xff, 0xff]);

    let dims: Vec<_> = p.mips.iter().map(|m| (m.width, m.height)).collect();
    assert_eq!(
        dims,
        vec![
            (512, 512),
            (256, 256),
            (128, 128),
            (64, 64),
            (32, 32),
            (16, 16),
            (8, 8),
            (4, 4),
        ]
    );

    // Large mips are LZO-compressed in this oracle (width high bit).
    assert!(
        bytes[p.sffo[0] as usize + 1] & 0x80 != 0,
        "mip0 should set LZO bit on width"
    );
    assert!(p.mips[0].lzo);
    assert!(p.mips[1].lzo);
    assert!(!p.mips[2].lzo);

    // Compressed mip0 must be smaller than raw BC3 size.
    let raw_bc3 = (512u32 / 4) * (512 / 4) * 16;
    assert!(p.mips[0].data.len() < raw_bc3 as usize);
}

fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(data);
    format!("{:x}", h.finalize())
}
