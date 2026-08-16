//! Repository-owned workflow invariants that protect the trusted/untrusted boundary.

use std::{fs, path::PathBuf};

#[test]
fn fork_opt_in_is_scoped_to_the_untrusted_proposal_checkout() {
    let workflow = fs::read_to_string(repository_root().join(".github/workflows/authority.yml"))
        .expect("read authority workflow");
    let trusted = section(
        &workflow,
        "- name: Check out trusted base",
        "- name: Isolate trusted",
    );
    let proposal = section(
        &workflow,
        "- name: Check out proposal as untrusted data",
        "- name: Independently review",
    );

    assert!(!trusted.contains("allow-unsafe-pr-checkout"));
    assert!(proposal.contains("persist-credentials: false"));
    assert!(proposal.contains("allow-unsafe-pr-checkout: true"));
    assert_eq!(
        workflow.matches("allow-unsafe-pr-checkout: true").count(),
        1
    );
}

#[test]
fn proposal_is_only_passed_to_the_trusted_review_binary() {
    let workflow = fs::read_to_string(repository_root().join(".github/workflows/authority.yml"))
        .expect("read authority workflow");
    let review = section(
        &workflow,
        "- name: Independently review observed proposal state",
        "__end_of_workflow__",
    );

    assert!(review.contains("$CARGO_TARGET_DIR/debug/zrail\" review"));
    assert!(review.contains("--root \"$GITHUB_WORKSPACE/proposal\""));
    assert!(!workflow.contains("cd proposal"));
    assert!(!workflow.contains("proposal/scripts/"));
}

fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start = source.find(start).expect("workflow section start");
    let tail = &source[start..];
    let end = tail.find(end).unwrap_or(tail.len());
    &tail[..end]
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("testkit lives below repository root")
        .to_path_buf()
}
