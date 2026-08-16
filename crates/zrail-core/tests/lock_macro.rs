//! Local macro observations are current-epoch, canonical resolved architecture.

use zrail_core::{LockFile, LockedMacroDefinition};

#[test]
fn macro_body_digest_participates_in_resolved_state() {
    let mut left = LockFile::new("0".repeat(64));
    let mut right = left.clone();
    left.macros.push(definition("a"));
    right.macros.push(definition("b"));

    assert!(!left.same_resolved_state(&right));
    assert!(
        left.render()
            .expect("render current macro state")
            .contains("[[macro]]")
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

fn definition(digit: &str) -> LockedMacroDefinition {
    LockedMacroDefinition {
        path: "src/lib.rs".into(),
        name: "local::reviewed".into(),
        ordinal: 1,
        sha256: digit.repeat(64),
    }
}
