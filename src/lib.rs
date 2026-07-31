//! Open-source PNG/TGA → Bohemia Interactive PAA encoder.
//!
//! Contract: [`docs/paa-profile.md`](../docs/paa-profile.md).

mod codec;
mod container;
mod encode;
mod error;
mod lzo;
mod metadata;
mod mipmaps;
mod options;

pub use container::{MipPayload, ParsedPaa, parse_paa};
pub use encode::{encode_rgba8, encode_rgba8_to};
pub use error::EncodeError;
pub use lzo::decompress_exact;
pub use options::{EncodeOptions, PaFormat};
