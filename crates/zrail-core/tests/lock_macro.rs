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

#[test]
fn released_macro_digest_spelling_remains_readable_for_migration() {
    let mut lock = LockFile::new("0".repeat(64));
    lock.macro_implementations.push(implementation("a"));
    let current = lock.render().unwrap();
    assert!(current.contains("inputs_sha256"));
    let previous = current
        .replace("inputs_sha256", "manifest_sha256")
        .replace("semantics = 5", "semantics = 4");
    let path = std::env::temp_dir().join(format!("zrail-legacy-macro-{}.lock", std::process::id()));
    std::fs::write(&path, &previous).unwrap();
    let parsed = LockFile::read(&path).unwrap();
    assert_eq!(parsed.macro_implementations, lock.macro_implementations);
    assert_eq!(parsed.semantics, 4);
    assert_eq!(parsed.render().unwrap(), previous);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn legacy_rendering_never_renames_text_inside_string_values() {
    let mut lock = LockFile::new("0".repeat(64));
    lock.semantics = 4;
    lock.analysis.as_mut().unwrap().analyzer_semantics = 4;
    lock.producer = "custom producer\ninputs_sha256 = preserved\n".into();
    lock.macro_implementations.push(implementation("a"));
    let rendered = lock.render().unwrap();
    let parsed: LockFile = toml::from_str(&rendered).unwrap();
    assert_eq!(parsed.producer, lock.producer);
    assert_eq!(parsed.macro_implementations, lock.macro_implementations);
    assert_eq!(parsed.render().unwrap(), rendered);
}

fn implementation(digit: &str) -> LockedMacroImplementation {
    LockedMacroImplementation {
        package: "fixture".into(),
        directory: ".".into(),
        inputs_sha256: digit.repeat(64),
    }
}
