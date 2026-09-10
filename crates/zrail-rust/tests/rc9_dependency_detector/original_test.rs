//! The complete original detector function is unchanged, including its inline input.

use super::super::lockfile_packages;

#[test]
fn package_extraction_matches_exact_names_only() {
    let packages = lockfile_packages(
        "[[package]]\nname = \"tokio-util-extra\"\n[[package]]\nname = \"bytes\"\n",
    );

    assert!(!packages.contains("tokio"));
    assert!(packages.contains("bytes"));
}

pub(super) fn check() {
    package_extraction_matches_exact_names_only();
}
