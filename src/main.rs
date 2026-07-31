//! Open CLI replacement for Bohemia Interactive's ImageToPAA.
//!
//! Goal: convert power-of-two RGBA sources (PNG/TGA) into `.paa` textures that
//! Arma 3 / TexView2 can load, including smooth alpha for UI silhouettes.

mod paa;

use anyhow::{bail, Context, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "image-to-paa",
    about = "Convert PNG/TGA images to Bohemia Interactive PAA textures",
    version
)]
struct Args {
    /// Input image (PNG or TGA). Prefer lossless RGBA with transparent silhouette.
    input: PathBuf,

    /// Output `.paa` path. Defaults to `<input>.paa`.
    output: Option<PathBuf>,

    /// Force DXT5 (explicit alpha). Default: DXT5 when the source has alpha,
    /// DXT1 otherwise.
    #[arg(long)]
    dxt5: bool,

    /// Force DXT1 (1-bit alpha / opaque). Ignored if `--dxt5` is set.
    #[arg(long)]
    dxt1: bool,

    /// Do not generate mipmaps.
    #[arg(long)]
    no_mips: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if !args.input.is_file() {
        bail!("input not found: {}", args.input.display());
    }

    let output = args.output.unwrap_or_else(|| {
        let mut path = args.input.clone();
        path.set_extension("paa");
        path
    });

    let img = image::open(&args.input)
        .with_context(|| format!("open {}", args.input.display()))?
        .to_rgba8();

    let (width, height) = img.dimensions();
    if !width.is_power_of_two() || !height.is_power_of_two() {
        bail!("source must be power-of-two (got {width}x{height}); pad/resize before converting");
    }

    let has_alpha = img.pixels().any(|px| px.0[3] < 255);
    let format = if args.dxt5 {
        paa::PaFormat::Dxt5
    } else if args.dxt1 {
        paa::PaFormat::Dxt1
    } else if has_alpha {
        paa::PaFormat::Dxt5
    } else {
        paa::PaFormat::Dxt1
    };

    let bytes = paa::encode_rgba8(&img, format, !args.no_mips).context("encode PAA")?;
    std::fs::write(&output, bytes).with_context(|| format!("write {}", output.display()))?;
    eprintln!(
        "Wrote {} ({format:?}, {width}x{height}, mips={})",
        output.display(),
        !args.no_mips
    );
    Ok(())
}
