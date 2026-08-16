//! Local macro observations are current-epoch, canonical resolved architecture.

use zrail_core::{LockFile, LockedMacroDefinition, LockedMacroImplementation};

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
fn legacy_epochs_cannot_smuggle_macro_authority() {
    let mut lock = LockFile::new("0".repeat(64));
    lock.schema = 4;
    lock.semantics = 4;
    lock.macros.push(definition("a"));

    let error = lock
        .render()
        .expect_err("legacy semantics cannot encode macro authority");
    assert!(error.to_string().contains("require lock semantics 5"));
}

#[test]
fn duplicate_macro_definition_identities_fail_closed() {
    let mut lock = LockFile::new("0".repeat(64));
    lock.schema = 5;
    lock.semantics = 5;
    lock.macros = vec![definition("a"), definition("b")];

    let error = lock
        .render()
        .expect_err("one identity cannot hold two body digests");
    assert!(
        error
            .to_string()
            .contains("duplicate locked macro definition")
    );
}

#[test]
fn current_epochs_reject_legacy_definition_authority() {
    let mut lock = LockFile::new("0".repeat(64));
    lock.macros.push(definition("a"));

    let error = lock
        .render()
        .expect_err("current locks require package manifests");
    assert!(error.to_string().contains("legacy semantics 5 state"));
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

fn definition(digit: &str) -> LockedMacroDefinition {
    LockedMacroDefinition {
        path: "src/lib.rs".into(),
        name: "local::reviewed".into(),
        ordinal: 1,
        sha256: digit.repeat(64),
    }
}

fn implementation(digit: &str) -> LockedMacroImplementation {
    LockedMacroImplementation {
        package: "fixture".into(),
        directory: ".".into(),
        manifest_sha256: digit.repeat(64),
    }
}
