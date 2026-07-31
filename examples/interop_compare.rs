//! Compare structural PAA metadata (not BCn payload identity).

use image_to_paa::parse_paa;
use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let (Some(ours), Some(official), label) = (args.next(), args.next(), args.next()) else {
        eprintln!("usage: interop_compare <ours.paa> <official.paa> [label]");
        return ExitCode::from(2);
    };
    let label = label.unwrap_or_else(|| ours.clone());

    let a = parse_paa(&fs::read(&ours).expect("read ours")).expect("parse ours");
    let b = parse_paa(&fs::read(&official).expect("read official")).expect("parse official");

    if a.format != b.format {
        eprintln!("FAIL {label}: format {:?} vs {:?}", a.format, b.format);
        return ExitCode::FAILURE;
    }
    if a.mips.len() != b.mips.len() {
        eprintln!(
            "FAIL {label}: mip count {} vs {}",
            a.mips.len(),
            b.mips.len()
        );
        return ExitCode::FAILURE;
    }
    for (i, (ma, mb)) in a.mips.iter().zip(b.mips.iter()).enumerate() {
        if ma.width != mb.width || ma.height != mb.height {
            eprintln!(
                "FAIL {label} mip{i}: dims {}x{} vs {}x{}",
                ma.width, ma.height, mb.width, mb.height
            );
            return ExitCode::FAILURE;
        }
    }
    let names_a: Vec<_> = a.tags.iter().map(|(n, _)| n.as_str()).collect();
    let names_b: Vec<_> = b.tags.iter().map(|(n, _)| n.as_str()).collect();
    // Official may include extra tags; require our canonical ones present.
    for required in ["CGVA", "CXAM", "SFFO"] {
        if !names_a.contains(&required) || !names_b.contains(&required) {
            eprintln!("FAIL {label}: missing tag {required} (ours={names_a:?} off={names_b:?})");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}
