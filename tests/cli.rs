//! CLI behaviour tests.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

fn bin() -> Command {
    Command::cargo_bin("image-to-paa").unwrap()
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/sources")
        .join(name)
}

#[test]
fn auto_opaque_writes_dxt1_header() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("out.paa");
    bin()
        .args([
            fixture("opaque_blocks_16.png")
                .as_os_str()
                .to_str()
                .unwrap(),
            out.to_str().unwrap(),
        ])
        .assert()
        .success();
    let bytes = fs::read(&out).unwrap();
    assert_eq!(&bytes[0..2], [0x01, 0xFF]);
}

#[test]
fn auto_alpha_writes_dxt5_header() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("out.paa");
    bin()
        .args([
            fixture("soft_radial_16.png").as_os_str().to_str().unwrap(),
            out.to_str().unwrap(),
            "--format",
            "auto",
        ])
        .assert()
        .success();
    let bytes = fs::read(&out).unwrap();
    assert_eq!(&bytes[0..2], [0x05, 0xFF]);
}

#[test]
fn conflict_dxt1_dxt5_exit_2() {
    bin()
        .args([
            fixture("opaque_blocks_16.png")
                .as_os_str()
                .to_str()
                .unwrap(),
            "--dxt1",
            "--dxt5",
        ])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn conflict_format_and_alias_exit_2() {
    bin()
        .args([
            fixture("opaque_blocks_16.png")
                .as_os_str()
                .to_str()
                .unwrap(),
            "--format",
            "dxt1",
            "--dxt5",
        ])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn soft_alpha_forced_dxt1_fails() {
    bin()
        .args([
            fixture("soft_radial_16.png").as_os_str().to_str().unwrap(),
            "--format",
            "dxt1",
        ])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("alpha"));
}

#[test]
fn refuses_overwrite_without_force() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("out.paa");
    fs::write(&out, b"keep-me").unwrap();
    bin()
        .args([
            fixture("opaque_blocks_16.png")
                .as_os_str()
                .to_str()
                .unwrap(),
            out.to_str().unwrap(),
        ])
        .assert()
        .failure();
    assert_eq!(fs::read(&out).unwrap(), b"keep-me");
}

#[test]
fn force_overwrites_atomically() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("out.paa");
    fs::write(&out, b"old").unwrap();
    bin()
        .args([
            fixture("opaque_blocks_16.png")
                .as_os_str()
                .to_str()
                .unwrap(),
            out.to_str().unwrap(),
            "--force",
        ])
        .assert()
        .success();
    let bytes = fs::read(&out).unwrap();
    assert_eq!(&bytes[0..2], [0x01, 0xFF]);
    assert!(!dir.path().join(".out.paa.tmp").exists());
}

#[test]
fn spaces_in_paths() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("my out file.paa");
    bin()
        .args([
            fixture("spaces in name 8.png")
                .as_os_str()
                .to_str()
                .unwrap(),
            out.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert!(out.is_file());
}

#[test]
fn missing_output_dir_fails() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("nope").join("out.paa");
    bin()
        .args([
            fixture("opaque_blocks_16.png")
                .as_os_str()
                .to_str()
                .unwrap(),
            out.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("directory"));
}

#[test]
fn non_paa_extension_rejected() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("out.bin");
    bin()
        .args([
            fixture("opaque_blocks_16.png")
                .as_os_str()
                .to_str()
                .unwrap(),
            out.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(".paa"));
}
