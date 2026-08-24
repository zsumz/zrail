//! Cargo.lock-resolved macro authorities require canonical immutable identities.

use super::LockedMacroSource;
use crate::LockFile;

#[test]
fn registry_macro_sources_require_and_render_exact_checksums() {
    let mut lock = LockFile::new("0".repeat(64));
    lock.macro_sources.push(LockedMacroSource {
        allowance: "serde::Serialize".into(),
        package: "serde_derive".into(),
        version: "1.0.229".into(),
        source: "registry+https://github.com/rust-lang/crates.io-index".into(),
        checksum: Some("1".repeat(64)),
    });

    let rendered = lock.render().expect("render exact macro source");

    assert!(rendered.contains("[[macro_source]]"));
    assert!(rendered.contains("allowance = \"serde::Serialize\""));
    assert!(rendered.contains("version = \"1.0.229\""));
}

#[test]
fn registry_macro_sources_without_checksums_fail_closed() {
    let mut lock = LockFile::new("0".repeat(64));
    lock.macro_sources.push(LockedMacroSource {
        allowance: "derive".into(),
        package: "derive-impl".into(),
        version: "1.2.3".into(),
        source: "registry+https://example.invalid/index".into(),
        checksum: None,
    });

    let error = lock.render().expect_err("checksum is authoritative");

    assert!(error.to_string().contains("requires a SHA-256 checksum"));
}
