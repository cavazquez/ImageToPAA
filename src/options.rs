//! Public encoding options and pixel format selection.

/// PAA compression type for the MVP DXT profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaFormat {
    /// Auto: DXT1 if fully opaque, DXT5 if any alpha < 255.
    Auto,
    /// Force DXT1/BC1. Source must be fully opaque.
    Dxt1,
    /// Force DXT5/BC3 with interpolated alpha.
    Dxt5,
}

/// Options for [`crate::encode_rgba8`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncodeOptions {
    pub format: PaFormat,
    /// When `false`, only the base mip is written.
    pub generate_mips: bool,
    /// When `true`, try per-mip LZO1X; keep raw BCn if LZO does not shrink.
    pub compress: bool,
}

impl Default for EncodeOptions {
    fn default() -> Self {
        Self {
            format: PaFormat::Auto,
            generate_mips: true,
            compress: false,
        }
    }
}
