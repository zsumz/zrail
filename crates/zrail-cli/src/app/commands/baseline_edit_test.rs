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

fn candidate(rule: &'static str, target: &str) -> BaselineRatchet {
    BaselineRatchet {
        rule,
        target: target.into(),
        reason: "Generated reason.",
    }
}
