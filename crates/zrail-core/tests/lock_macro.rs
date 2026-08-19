//! Local macro observations are current-epoch, canonical resolved architecture.

use zrail_core::{LockFile, LockedMacroImplementation};

#[test]
fn macro_package_digest_participates_in_resolved_state() {
    let mut left = LockFile::new("0".repeat(64));
    let mut right = left.clone();
    left.macro_implementations.push(implementation("a"));
    right.macro_implementations.push(implementation("b"));

    assert!(!left.same_resolved_state(&right));
    assert!(
        left.render()
            .expect("render current macro state")
            .contains("[[macro_implementation]]")
    );
}

#[test]
fn duplicate_macro_package_identities_fail_closed() {
    let mut lock = LockFile::new("0".repeat(64));
    lock.macro_implementations = vec![implementation("a"), implementation("b")];

    let error = lock
        .render()
        .expect_err("one package has one implementation manifest");
    assert!(
        error
            .to_string()
            .contains("duplicate locked macro implementation")
    );
}

fn implementation(digit: &str) -> LockedMacroImplementation {
    LockedMacroImplementation {
        package: "fixture".into(),
        directory: ".".into(),
        manifest_sha256: digit.repeat(64),
    }
}
