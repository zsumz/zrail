//! Deterministic repository macro implementation manifests.

use std::collections::BTreeMap;

use super::{MAX_IMPLEMENTATION_INPUTS, digest_inputs};

#[test]
fn manifest_digest_is_path_and_content_bound() {
    let inputs = BTreeMap::from([
        ("Cargo.toml".into(), b"[package]\nname='macros'\n".to_vec()),
        ("src/lib.rs".into(), b"pub fn expand() {}\n".to_vec()),
    ]);
    let original = digest_inputs(&inputs).expect("digest inputs");

    let mut changed_content = inputs.clone();
    changed_content.insert(
        "src/lib.rs".into(),
        b"pub fn expand() { helper() }\n".to_vec(),
    );
    let mut changed_path = inputs;
    let source = changed_path.remove("src/lib.rs").expect("source input");
    changed_path.insert("src/expand.rs".into(), source);

    assert_ne!(
        original,
        digest_inputs(&changed_content).expect("content digest")
    );
    assert_ne!(original, digest_inputs(&changed_path).expect("path digest"));
}

#[test]
fn manifest_digest_is_deterministic() {
    let inputs = BTreeMap::from([
        ("src/z.rs".into(), b"z\n".to_vec()),
        ("src/a.rs".into(), b"a\n".to_vec()),
    ]);

    assert_eq!(
        digest_inputs(&inputs).expect("first digest"),
        digest_inputs(&inputs).expect("second digest")
    );
}

#[test]
fn manifest_input_count_is_bounded() {
    let inputs = (0..=MAX_IMPLEMENTATION_INPUTS)
        .map(|index| (format!("src/{index}.rs"), Vec::new()))
        .collect();

    let error = digest_inputs(&inputs).expect_err("excessive input count must fail");

    assert!(error.to_string().contains("input safety limit"));
}
