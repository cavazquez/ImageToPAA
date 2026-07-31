//! Decode PAA base mip and compose over white/black for silhouette QA.
//!
//! ```bash
//! cargo run --release --example compose_paa_qa -- \
//!   /path/to/r210_west.paa /path/to/out-dir
//! ```

use image::{Rgba, RgbaImage};
use image_to_paa::{parse_paa, PaFormat};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use texpresso::Format;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(paa_path) = args.next() else {
        eprintln!("usage: compose_paa_qa <file.paa|dir> [out-dir]");
        return ExitCode::from(2);
    };
    let out_dir = PathBuf::from(args.next().unwrap_or_else(|| "paa-qa-out".into()));
    let _ = fs::create_dir_all(&out_dir);

    let paths: Vec<PathBuf> = {
        let p = PathBuf::from(&paa_path);
        if p.is_dir() {
            let mut v: Vec<_> = fs::read_dir(&p)
                .expect("read dir")
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("paa"))
                .collect();
            v.sort();
            v
        } else {
            vec![p]
        }
    };

    let mut failed = 0usize;
    for path in paths {
        match qa_one(&path, &out_dir) {
            Ok(report) => println!("{report}"),
            Err(e) => {
                eprintln!("FAIL {}: {e}", path.display());
                failed += 1;
            }
        }
    }
    if failed > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn qa_one(path: &Path, out_dir: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let parsed = parse_paa(&bytes)?;
    if parsed.format != PaFormat::Dxt5 {
        return Err(format!("expected DXT5, got {:?}", parsed.format));
    }
    let mip = parsed.mips.first().ok_or("no mips")?;
    let w = u32::from(mip.width);
    let h = u32::from(mip.height);
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    Format::Bc3.decompress(&mip.data, w as usize, h as usize, &mut rgba);

    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("out");

    let img = RgbaImage::from_raw(w, h, rgba.clone()).ok_or("rgba")?;
    img.save(out_dir.join(format!("{stem}_decoded.png")))
        .map_err(|e| e.to_string())?;
    compose(&img, [255, 255, 255])
        .save(out_dir.join(format!("{stem}_on_white.png")))
        .map_err(|e| e.to_string())?;
    compose(&img, [0, 0, 0])
        .save(out_dir.join(format!("{stem}_on_black.png")))
        .map_err(|e| e.to_string())?;

    let (halo_w, halo_b, soft, opaque, clear) = edge_stats(&rgba, w, h);
    if soft == 0 {
        return Err("no intermediate alpha (not soft silhouette?)".into());
    }
    if clear == 0 {
        return Err("no fully transparent pixels".into());
    }
    if opaque == 0 {
        return Err("no opaque pixels".into());
    }

    // Report fringe metrics for human/TexView follow-up. Dark RGB under partial
    // alpha is common in these masters (authored on black); do not hard-fail.
    let warn = if halo_w > 12.0 || halo_b > 12.0 {
        " WARN_fringe"
    } else {
        ""
    };

    Ok(format!(
        "OK{warn} {stem}: {w}x{h} mips={} softα={soft} opaque={opaque} clear={clear} fringeW={halo_w:.1} fringeB={halo_b:.1}",
        parsed.mips.len()
    ))
}

fn compose(src: &RgbaImage, bg: [u8; 3]) -> RgbaImage {
    let (w, h) = src.dimensions();
    let mut out = RgbaImage::new(w, h);
    for (x, y, px) in src.enumerate_pixels() {
        let a = px.0[3] as f32 / 255.0;
        let r = (px.0[0] as f32 * a + bg[0] as f32 * (1.0 - a)).round() as u8;
        let g = (px.0[1] as f32 * a + bg[1] as f32 * (1.0 - a)).round() as u8;
        let b = (px.0[2] as f32 * a + bg[2] as f32 * (1.0 - a)).round() as u8;
        out.put_pixel(x, y, Rgba([r, g, b, 255]));
    }
    out
}

/// Measure RGB contribution of nearly-transparent edge-ish pixels after un-premultiply.
fn edge_stats(rgba: &[u8], w: u32, h: u32) -> (f64, f64, usize, usize, usize) {
    let mut soft = 0usize;
    let mut opaque = 0usize;
    let mut clear = 0usize;
    let mut sum_w = 0.0f64;
    let mut sum_b = 0.0f64;
    let mut n_edge = 0usize;

    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let a = rgba[i + 3];
            match a {
                0 => clear += 1,
                255 => opaque += 1,
                _ => soft += 1,
            }
            // Near-transparent with non-zero RGB is the classic matte/halo source.
            if a > 0 && a < 40 {
                let r = rgba[i] as f64;
                let g = rgba[i + 1] as f64;
                let b = rgba[i + 2] as f64;
                // Composited against white: dark RGB under low A pulls toward black.
                let cw = (r * (a as f64) + 255.0 * (255.0 - a as f64)) / 255.0;
                let cg = (g * (a as f64) + 255.0 * (255.0 - a as f64)) / 255.0;
                let cb = (b * (a as f64) + 255.0 * (255.0 - a as f64)) / 255.0;
                let mae_w = ((255.0 - cw) + (255.0 - cg) + (255.0 - cb)) / 3.0;
                // Against black: bright RGB under low A pulls toward white.
                let dw = r * a as f64 / 255.0;
                let dg = g * a as f64 / 255.0;
                let db = b * a as f64 / 255.0;
                let mae_b = (dw + dg + db) / 3.0;
                sum_w += mae_w;
                sum_b += mae_b;
                n_edge += 1;
            }
        }
    }
    let halo_w = if n_edge > 0 {
        sum_w / n_edge as f64
    } else {
        0.0
    };
    let halo_b = if n_edge > 0 {
        sum_b / n_edge as f64
    } else {
        0.0
    };
    (halo_w, halo_b, soft, opaque, clear)
}
