//! Selector-aware ratchets have canonical, collision-safe lock identities.

use zrail_core::{LockFile, LockedRatchet};

#[test]
fn render_normalizes_selector_spelling() {
    let mut lock = LockFile::new("0".repeat(64));
    lock.ratchets.push(ratchet("r#unwrap"));

    let rendered = lock.render().expect("render selector ratchet");

    assert!(rendered.contains("selector = \"unwrap\""));
}

#[test]
fn normalized_selector_collisions_are_rejected() {
    let mut lock = LockFile::new("0".repeat(64));
    lock.ratchets = vec![ratchet("unwrap"), ratchet("r#unwrap")];

    let error = lock.render().expect_err("duplicate identity must fail");

    assert!(error.to_string().contains("duplicate locked ratchet"));
}

fn ratchet(selector: &str) -> LockedRatchet {
    LockedRatchet {
        rule: "rust.hygiene.denied-method".into(),
        selector: Some(selector.into()),
        target: "src/lib.rs".into(),
        value: 1,
    }
}
