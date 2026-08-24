//! Baseline contract edits preserve human text and are idempotent.

use zrail_core::RatchetContract;
use zrail_rust::BaselineRatchet;

use super::merge;

#[test]
fn appends_only_missing_ratchets_without_reserializing_other_text() {
    let source = concat!(
        "# human heading\n",
        "schema = 1 # keep me\n\n",
        "[[ratchet]]\n",
        "rule = \"rust.file-size\"\n",
        "target = \"src/old.rs\"\n",
        "reason = \"Human reason.\"\n",
    );
    let existing = vec![RatchetContract {
        rule: "rust.file-size".into(),
        selector: None,
        target: "src/old.rs".into(),
        reason: "Human reason.".into(),
    }];
    let candidates = vec![
        candidate("rust.file-size", "src/old.rs"),
        candidate("rust.file-size", "src/new.rs"),
    ];

    let edit = merge(source, &existing, candidates);

    assert!(edit.contract.starts_with(source));
    assert!(edit.contract.contains("target = \"src/new.rs\""));
    assert_eq!(edit.contract.matches("Human reason.").count(), 1);
    assert_eq!(edit.added.len(), 1);
    assert_eq!(edit.preserved.len(), 1);
}

#[test]
fn no_additions_leave_contract_bytes_exactly_unchanged() {
    let source = "# deliberate trailing spaces  \nschema = 1\n";
    let existing = vec![RatchetContract {
        rule: "rust.file-size".into(),
        selector: None,
        target: "src/lib.rs".into(),
        reason: "Human reason.".into(),
    }];

    let edit = merge(
        source,
        &existing,
        vec![candidate("rust.file-size", "src/lib.rs")],
    );

    assert_eq!(edit.contract, source);
}

#[test]
fn selector_ratchets_render_and_compare_by_normalized_identity() {
    let source = "schema = 1\n";
    let existing = vec![RatchetContract {
        rule: "rust.hygiene.denied-method".into(),
        selector: Some("r#unwrap".into()),
        target: "src/lib.rs".into(),
        reason: "Human reason.".into(),
    }];
    let mut normalized = candidate("rust.hygiene.denied-method", "src/lib.rs");
    normalized.selector = Some("unwrap".into());

    let preserved = merge(source, &existing, vec![normalized.clone()]);
    assert_eq!(preserved.contract, source);
    assert_eq!(preserved.preserved.len(), 1);

    let added = merge(source, &[], vec![normalized]);
    assert!(added.contract.contains("selector = \"unwrap\""));
}

fn candidate(rule: &'static str, target: &str) -> BaselineRatchet {
    BaselineRatchet {
        rule,
        selector: None,
        target: target.into(),
        reason: "Generated reason.",
    }
}
