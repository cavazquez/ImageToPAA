//! Binary PAA container serializer (independent of BCn compression).

use crate::error::EncodeError;
use crate::metadata::Tag;
use crate::options::PaFormat;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct MipPayload {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}

/// Serialize a complete PAA file from prepared tags + mip payloads.
/// Inserts `SFFO` after the semantic tags (caller must NOT include SFFO).
pub fn write_paa<W: Write>(
    mut w: W,
    format: PaFormat,
    semantic_tags: &[Tag],
    mips: &[MipPayload],
) -> Result<(), EncodeError> {
    if !matches!(format, PaFormat::Dxt1 | PaFormat::Dxt5) {
        return Err(EncodeError::Write(
            "container requires resolved Dxt1 or Dxt5".into(),
        ));
    }
    if mips.is_empty() {
        return Err(EncodeError::Write("at least one mip required".into()));
    }
    if mips.len() > 16 {
        return Err(EncodeError::TooManyMips { count: mips.len() });
    }
    for mip in mips {
        if mip.data.len() > 0xFF_FFFF {
            return Err(EncodeError::PayloadTooLarge {
                len: mip.data.len(),
            });
        }
    }

    let type_bytes: [u8; 2] = match format {
        PaFormat::Dxt1 => [0x01, 0xFF],
        PaFormat::Dxt5 => [0x05, 0xFF],
        PaFormat::Auto => unreachable!(),
    };

    let mut cursor: u32 = 2;
    for tag in semantic_tags {
        cursor = cursor
            .checked_add(12)
            .and_then(|c| c.checked_add(tag.payload.len() as u32))
            .ok_or_else(|| EncodeError::Write("tag size overflow".into()))?;
    }
    cursor = cursor
        .checked_add(76)
        .and_then(|c| c.checked_add(2))
        .ok_or_else(|| EncodeError::Write("header size overflow".into()))?;

    let mut sffo = [0u8; 64];
    let mut off = cursor;
    for (i, mip) in mips.iter().enumerate() {
        sffo[i * 4..i * 4 + 4].copy_from_slice(&off.to_le_bytes());
        let step = 2u32 + 2 + 3 + mip.data.len() as u32;
        off = off
            .checked_add(step)
            .ok_or_else(|| EncodeError::Write("mip offset overflow".into()))?;
    }

    w.write_all(&type_bytes)
        .map_err(|e| EncodeError::Write(e.to_string()))?;

    for tag in semantic_tags {
        write_tag(&mut w, &tag.name, &tag.payload)?;
    }
    write_tag(&mut w, b"SFFO", &sffo)?;

    w.write_all(&[0, 0])
        .map_err(|e| EncodeError::Write(e.to_string()))?;

    for mip in mips {
        w.write_all(&mip.width.to_le_bytes())
            .map_err(|e| EncodeError::Write(e.to_string()))?;
        w.write_all(&mip.height.to_le_bytes())
            .map_err(|e| EncodeError::Write(e.to_string()))?;
        let len = mip.data.len() as u32;
        let len_bytes = len.to_le_bytes();
        w.write_all(&len_bytes[..3])
            .map_err(|e| EncodeError::Write(e.to_string()))?;
        w.write_all(&mip.data)
            .map_err(|e| EncodeError::Write(e.to_string()))?;
    }

    w.write_all(&[0, 0, 0, 0, 0, 0])
        .map_err(|e| EncodeError::Write(e.to_string()))?;
    Ok(())
}

fn write_tag<W: Write>(w: &mut W, name: &[u8; 4], payload: &[u8]) -> Result<(), EncodeError> {
    w.write_all(b"GGAT")
        .map_err(|e| EncodeError::Write(e.to_string()))?;
    w.write_all(name)
        .map_err(|e| EncodeError::Write(e.to_string()))?;
    w.write_all(&(payload.len() as u32).to_le_bytes())
        .map_err(|e| EncodeError::Write(e.to_string()))?;
    w.write_all(payload)
        .map_err(|e| EncodeError::Write(e.to_string()))?;
    Ok(())
}

#[derive(Debug)]
pub struct ParsedPaa {
    pub format: PaFormat,
    pub tags: Vec<(String, Vec<u8>)>,
    pub mips: Vec<MipPayload>,
    pub sffo: [u32; 16],
}

/// Structural parser used by tests and tooling.
pub fn parse_paa(data: &[u8]) -> Result<ParsedPaa, String> {
    if data.len() < 4 {
        return Err("too short".into());
    }
    let format = match &data[0..2] {
        [0x01, 0xFF] => PaFormat::Dxt1,
        [0x05, 0xFF] => PaFormat::Dxt5,
        other => return Err(format!("bad type {other:?}")),
    };
    let mut i = 2usize;
    let mut tags = Vec::new();
    let mut sffo = [0u32; 16];

    while i + 12 <= data.len() && &data[i..i + 4] == b"GGAT" {
        let name = String::from_utf8_lossy(&data[i + 4..i + 8]).into_owned();
        let len = u32::from_le_bytes(data[i + 8..i + 12].try_into().unwrap()) as usize;
        i += 12;
        if i + len > data.len() {
            return Err("tag payload OOB".into());
        }
        let payload = data[i..i + len].to_vec();
        if name == "SFFO" {
            if payload.len() != 64 {
                return Err("SFFO len".into());
            }
            for t in 0..16 {
                sffo[t] = u32::from_le_bytes(payload[t * 4..t * 4 + 4].try_into().unwrap());
            }
        }
        tags.push((name, payload));
        i += len;
    }

    if i + 2 > data.len() {
        return Err("missing palette".into());
    }
    i += 2;

    let mut mips = Vec::new();
    for (m, &expected_u32) in sffo.iter().take_while(|&&o| o != 0).enumerate() {
        let expected = expected_u32 as usize;
        if i != expected {
            return Err(format!("mip {m} offset got {i} expected {expected}"));
        }
        if i + 7 > data.len() {
            return Err("mip header OOB".into());
        }
        let width = u16::from_le_bytes(data[i..i + 2].try_into().unwrap()) & 0x7FFF;
        let height = u16::from_le_bytes(data[i + 2..i + 4].try_into().unwrap());
        let mut len_b = [0u8; 4];
        len_b[..3].copy_from_slice(&data[i + 4..i + 7]);
        let len = u32::from_le_bytes(len_b) as usize;
        i += 7;
        if i + len > data.len() {
            return Err("mip data OOB".into());
        }
        mips.push(MipPayload {
            width,
            height,
            data: data[i..i + len].to_vec(),
        });
        i += len;
    }

    Ok(ParsedPaa {
        format,
        tags,
        mips,
        sffo,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::Tag;

    fn artificial_mips(n: usize) -> Vec<MipPayload> {
        (0..n)
            .map(|i| MipPayload {
                width: (16 >> i.min(2)) as u16,
                height: (16 >> i.min(2)) as u16,
                data: vec![0xAB; 8],
            })
            .collect()
    }

    #[test]
    fn sffo_points_at_mip_widths() {
        let tags = vec![
            Tag::new(b"CGVA", vec![1, 2, 3, 4]),
            Tag::new(b"CXAM", vec![255; 4]),
        ];
        let mips = artificial_mips(2);
        let mut buf = Vec::new();
        write_paa(&mut buf, PaFormat::Dxt1, &tags, &mips).unwrap();
        let parsed = parse_paa(&buf).unwrap();
        assert_eq!(parsed.format, PaFormat::Dxt1);
        assert_eq!(parsed.mips.len(), 2);
        assert_eq!(parsed.sffo[2..], [0; 14]);
        for (i, mip) in parsed.mips.iter().enumerate() {
            assert_eq!(mip.data, vec![0xAB; 8]);
            let off = parsed.sffo[i] as usize;
            let w = u16::from_le_bytes(buf[off..off + 2].try_into().unwrap());
            assert_eq!(w, mip.width);
        }
    }

    #[test]
    fn one_and_sixteen_mips() {
        let tags = [
            Tag::new(b"CGVA", vec![0; 4]),
            Tag::new(b"CXAM", vec![255; 4]),
        ];
        for n in [1usize, 16] {
            let mips = artificial_mips(n);
            let mut buf = Vec::new();
            write_paa(&mut buf, PaFormat::Dxt1, &tags, &mips).unwrap();
            let p = parse_paa(&buf).unwrap();
            assert_eq!(p.mips.len(), n);
            assert!(p.sffo[n..].iter().all(|&x| x == 0));
        }
    }

    #[test]
    fn rejects_seventeen_mips() {
        let tags = [
            Tag::new(b"CGVA", vec![0; 4]),
            Tag::new(b"CXAM", vec![255; 4]),
        ];
        let mips = artificial_mips(17);
        let err = write_paa(Vec::new(), PaFormat::Dxt1, &tags, &mips).unwrap_err();
        assert!(matches!(err, EncodeError::TooManyMips { count: 17 }));
    }

    #[test]
    fn rejects_payload_over_u24() {
        let tags = [
            Tag::new(b"CGVA", vec![0; 4]),
            Tag::new(b"CXAM", vec![255; 4]),
        ];
        let mips = vec![MipPayload {
            width: 4,
            height: 4,
            data: vec![0; 0x0100_0000],
        }];
        let err = write_paa(Vec::new(), PaFormat::Dxt5, &tags, &mips).unwrap_err();
        assert!(matches!(err, EncodeError::PayloadTooLarge { .. }));
    }

    #[test]
    fn structural_fixture_dxt5_galf() {
        let tags = vec![
            Tag::new(b"CGVA", vec![0, 0, 0, 128]),
            Tag::new(b"CXAM", vec![255; 4]),
            Tag::new(b"GALF", vec![1, 0, 0, 0]),
        ];
        let mips = vec![MipPayload {
            width: 4,
            height: 4,
            data: vec![0x11; 16],
        }];
        let mut buf = Vec::new();
        write_paa(&mut buf, PaFormat::Dxt5, &tags, &mips).unwrap();
        assert_eq!(&buf[0..2], [0x05, 0xFF]);
        assert!(buf.windows(4).any(|w| w == b"GALF"));
        assert_eq!(buf.len(), 2 + 16 * 3 + 76 + 2 + 7 + 16 + 6);
        let p = parse_paa(&buf).unwrap();
        let names: Vec<_> = p.tags.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["CGVA", "CXAM", "GALF", "SFFO"]);
    }
}
