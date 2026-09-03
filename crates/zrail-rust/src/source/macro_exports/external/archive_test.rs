//! Hostile archive paths and lock identities fail before cache extraction.

use std::path::Path;

use crate::cargo::ResolvedPackageIdentity;

use super::{archive_name, checked_member_path, normalized_relative};

#[test]
fn archive_paths_reject_escape_and_noncanonical_components() {
    assert_eq!(
        checked_member_path(Path::new("sample-1.0.0/src/lib.rs"), "sample-1.0.0")
            .expect("valid archive path"),
        "src/lib.rs"
    );
    for path in [
        "sample-1.0.0/../outside.rs",
        "/sample-1.0.0/src/lib.rs",
        "other-1.0.0/src/lib.rs",
        "sample-1.0.0/src\\lib.rs",
    ] {
        assert!(
            checked_member_path(Path::new(path), "sample-1.0.0").is_err(),
            "accepted hostile archive path {path:?}"
        );
    }
    for path in ["../src/lib.rs", "/src/lib.rs", "src/./lib.rs", ""] {
        assert!(
            normalized_relative(path).is_err(),
            "accepted noncanonical source path {path:?}"
        );
    }
}

#[test]
fn archive_filename_rejects_path_syntax_from_lock_data() {
    let identity = ResolvedPackageIdentity {
        name: "../sample".into(),
        version: "1.0.0".into(),
        source: "registry+https://example.invalid/index".into(),
        checksum: Some("0".repeat(64)),
    };
    assert!(archive_name(&identity).is_err());
}
