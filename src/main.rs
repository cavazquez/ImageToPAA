//! CLI for image-to-paa.

use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};
use image_to_paa::{encode_rgba8, EncodeError, EncodeOptions, PaFormat};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum FormatArg {
    Auto,
    Dxt1,
    Dxt5,
}

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

    /// Compression format (`auto` chooses DXT1 if opaque, DXT5 if any alpha < 255).
    #[arg(long, value_enum)]
    format: Option<FormatArg>,

    /// Alias of `--format dxt5`. Conflicts with `--dxt1` and an explicit `--format`.
    #[arg(long)]
    dxt5: bool,

    /// Alias of `--format dxt1`. Conflicts with `--dxt5` and an explicit `--format`.
    #[arg(long)]
    dxt1: bool,

    /// Do not generate mipmaps.
    #[arg(long)]
    no_mips: bool,

    /// Overwrite an existing output file.
    #[arg(long)]
    force: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            let code = if err.downcast_ref::<UsageError>().is_some() || is_usage_error(&err) {
                2
            } else {
                1
            };
            eprintln!("error: {err:#}");
            ExitCode::from(code)
        }
    }
}

#[derive(Debug)]
struct UsageError(String);

impl std::fmt::Display for UsageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for UsageError {}

fn usage(msg: impl Into<String>) -> anyhow::Error {
    anyhow::Error::new(UsageError(msg.into()))
}

fn is_usage_error(err: &anyhow::Error) -> bool {
    err.chain().any(|e| {
        e.downcast_ref::<clap::Error>()
            .map(|c| {
                matches!(
                    c.kind(),
                    clap::error::ErrorKind::ArgumentConflict
                        | clap::error::ErrorKind::MissingRequiredArgument
                        | clap::error::ErrorKind::InvalidValue
                        | clap::error::ErrorKind::UnknownArgument
                )
            })
            .unwrap_or(false)
    })
}

fn run() -> Result<()> {
    let args = match Args::try_parse() {
        Ok(a) => a,
        Err(e) => {
            e.print().ok();
            if matches!(
                e.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) {
                return Ok(());
            }
            let code = if matches!(
                e.kind(),
                clap::error::ErrorKind::ArgumentConflict
                    | clap::error::ErrorKind::MissingRequiredArgument
                    | clap::error::ErrorKind::InvalidValue
                    | clap::error::ErrorKind::UnknownArgument
            ) {
                2
            } else {
                1
            };
            std::process::exit(code);
        }
    };

    if !args.input.is_file() {
        bail!(
            "input not found: {} (expected an existing PNG/TGA file)",
            args.input.display()
        );
    }

    let output = args.output.clone().unwrap_or_else(|| {
        let mut path = args.input.clone();
        path.set_extension("paa");
        path
    });

    if output
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("paa"))
        != Some(true)
    {
        bail!(
            "output must use a .paa extension (got {}); refusing to write non-PAA paths",
            output.display()
        );
    }

    if same_path(&args.input, &output)? {
        bail!(
            "input and output resolve to the same path ({}); refusing to overwrite the source",
            output.display()
        );
    }

    if output.exists() && !args.force {
        bail!(
            "output already exists: {} (pass --force to overwrite)",
            output.display()
        );
    }

    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    if !parent.as_os_str().is_empty() && !parent.exists() {
        bail!(
            "output directory does not exist: {} (create it before writing {})",
            parent.display(),
            output.display()
        );
    }

    let img = image::open(&args.input)
        .with_context(|| format!("open {}", args.input.display()))?
        .to_rgba8();
    let (width, height) = img.dimensions();

    if args.dxt1 && args.dxt5 {
        return Err(usage(
            "conflicting flags: --dxt1 and --dxt5 (use --format or a single alias)",
        ));
    }
    if args.format.is_some() && (args.dxt1 || args.dxt5) {
        return Err(usage(
            "conflicting flags: --format cannot be combined with --dxt1/--dxt5",
        ));
    }

    let format = if args.dxt5 {
        PaFormat::Dxt5
    } else if args.dxt1 {
        PaFormat::Dxt1
    } else {
        match args.format.unwrap_or(FormatArg::Auto) {
            FormatArg::Auto => PaFormat::Auto,
            FormatArg::Dxt1 => PaFormat::Dxt1,
            FormatArg::Dxt5 => PaFormat::Dxt5,
        }
    };

    let options = EncodeOptions {
        format,
        generate_mips: !args.no_mips,
    };

    let bytes =
        encode_rgba8(&img, options).map_err(|e| map_encode_error(e, &args.input, width, height))?;

    write_atomic(&output, &bytes)?;

    let resolved = match format {
        PaFormat::Auto => {
            if img.pixels().any(|p| p.0[3] < 255) {
                "DXT5"
            } else {
                "DXT1"
            }
        }
        PaFormat::Dxt1 => "DXT1",
        PaFormat::Dxt5 => "DXT5",
    };

    eprintln!(
        "Wrote {} ({resolved}, {width}x{height}, mips={})",
        output.display(),
        !args.no_mips
    );
    Ok(())
}

fn map_encode_error(err: EncodeError, path: &Path, width: u32, height: u32) -> anyhow::Error {
    match err {
        EncodeError::NotPowerOfTwo { width, height } => anyhow::anyhow!(
            "{}: dimensions {width}x{height} are not power-of-two (profile forbids implicit resize)",
            path.display()
        ),
        EncodeError::TooSmall { width, height } => anyhow::anyhow!(
            "{}: dimensions {width}x{height} below minimum 4x4",
            path.display()
        ),
        EncodeError::TooLarge { width, height } => anyhow::anyhow!(
            "{}: dimensions {width}x{height} exceed maximum 4096 per axis",
            path.display()
        ),
        EncodeError::NotBlockAligned { width, height } => anyhow::anyhow!(
            "{}: dimensions {width}x{height} are not multiples of 4 (BCn block)",
            path.display()
        ),
        EncodeError::SoftAlphaForcedDxt1 => anyhow::anyhow!(
            "{}: refuses --format dxt1 / --dxt1 because the source has alpha < 255 ({width}x{height}); use dxt5 or auto",
            path.display()
        ),
        other => anyhow::Error::new(other),
    }
}

fn same_path(a: &Path, b: &Path) -> Result<bool> {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(ca), Ok(cb)) => Ok(ca == cb),
        _ => Ok(a == b),
    }
}

fn write_atomic(output: &Path, bytes: &[u8]) -> Result<()> {
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let stem = output
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("out.paa");
    let tmp_name = format!(".{stem}.{}.tmp", std::process::id());
    let tmp_path = parent.join(tmp_name);

    let write_result = (|| {
        let mut f = File::create(&tmp_path)
            .with_context(|| format!("create temporary {}", tmp_path.display()))?;
        f.write_all(bytes)
            .with_context(|| format!("write temporary {}", tmp_path.display()))?;
        f.flush()
            .with_context(|| format!("flush temporary {}", tmp_path.display()))?;
        f.sync_all()
            .with_context(|| format!("sync temporary {}", tmp_path.display()))?;
        fs::rename(&tmp_path, output)
            .with_context(|| format!("publish {} from temporary", output.display()))?;
        Ok(())
    })();

    if write_result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }
    write_result
}
