//! Typed errors for the encoding core (no I/O).

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EncodeError {
    #[error("empty image")]
    EmptyImage,

    #[error("dimensions {width}x{height} are not power-of-two")]
    NotPowerOfTwo { width: u32, height: u32 },

    #[error("dimensions {width}x{height} below minimum 4x4")]
    TooSmall { width: u32, height: u32 },

    #[error("dimensions {width}x{height} exceed maximum 4096 per axis")]
    TooLarge { width: u32, height: u32 },

    #[error("dimensions {width}x{height} are not multiples of 4 (BCn block)")]
    NotBlockAligned { width: u32, height: u32 },

    #[error("DXT1 requires a fully opaque source; found alpha < 255")]
    SoftAlphaForcedDxt1,

    #[error("too many mip levels ({count}); maximum is 16")]
    TooManyMips { count: usize },

    #[error("mip payload length {len} exceeds u24 maximum")]
    PayloadTooLarge { len: usize },

    #[error("write failed: {0}")]
    Write(String),
}
