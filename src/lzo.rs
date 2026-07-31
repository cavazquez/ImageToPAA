//! Optional per-mip LZO1X via `lzokay` (MIT).
//!
//! Used only when `--compress` / `EncodeOptions::compress` is set. A mip is
//! stored LZO-compressed only if the result is strictly smaller than raw BCn.

use crate::error::EncodeError;

/// Compress `raw` with LZO1X. Returns `(payload, used_lzo)`.
/// On failure or no size win, returns the original bytes and `false`.
pub fn maybe_compress(raw: Vec<u8>) -> Result<(Vec<u8>, bool), EncodeError> {
    match lzokay::compress::compress(&raw) {
        Ok(comp) if comp.len() < raw.len() => Ok((comp, true)),
        Ok(_) => Ok((raw, false)),
        Err(e) => Err(EncodeError::Write(format!("LZO compress failed: {e}"))),
    }
}

/// Decompress LZO1X into a buffer of known uncompressed size (independent decoder).
pub fn decompress_exact(src: &[u8], uncompressed_len: usize) -> Result<Vec<u8>, EncodeError> {
    let mut dst = vec![0u8; uncompressed_len];
    let written = lzokay::decompress::decompress(src, &mut dst)
        .map_err(|e| EncodeError::Write(format!("LZO decompress failed: {e}")))?;
    if written != uncompressed_len {
        return Err(EncodeError::Write(format!(
            "LZO decompress size mismatch: got {written}, expected {uncompressed_len}"
        )));
    }
    Ok(dst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compressible_payload_shrinks() {
        let raw = vec![0u8; 4096];
        let (out, used) = maybe_compress(raw.clone()).unwrap();
        assert!(used);
        assert!(out.len() < raw.len());
        let round = decompress_exact(&out, raw.len()).unwrap();
        assert_eq!(round, raw);
    }

    #[test]
    fn tiny_or_random_may_stay_raw() {
        // 8-byte BC1 block: LZO almost always expands.
        let raw = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let (out, used) = maybe_compress(raw.clone()).unwrap();
        if used {
            assert!(out.len() < raw.len());
            assert_eq!(decompress_exact(&out, raw.len()).unwrap(), raw);
        } else {
            assert_eq!(out, raw);
        }
    }
}
