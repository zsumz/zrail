//! Protected review sees scoped permission changes without trusting lock replacement.

use crate::diff::compare_fixture_test::contract_with_hard_limit;
use crate::{ChangeKind, Contract, RatchetContract, SizePolicyContract, SizeTargetMode};

use super::compare;

fn configured() -> Contract {
    let mut contract = contract_with_hard_limit(500);
    contract.source.rust.budgets = Some(
        toml::from_str::<SizePolicyContract>(
            r#"
[[overrides]]
name = "core"
include = ["src/**"]
roles = ["implementation", "test"]
budget = { target = 240, soft = 300, hard = 500 }
reason = "Keep the core reviewable."
[[exceptions]]
path = "src/debt.rs"
hard = 550
reason = "Split this seam."
owner = "architecture"
issue = "ARCH-42"
"#,
        )
        .expect("policy"),
    );
    contract.ratchets.push(RatchetContract {
        rule: "rust.file-size".into(),
        selector: None,
        target: "src/debt.rs".into(),
        reason: "Measured existing debt.".into(),
        baseline: Some(530),
    });
    contract
}

#[test]
fn higher_limits_and_warning_downgrades_are_grants() {
    let before = configured();
    for field in ["target", "soft", "hard", "mode"] {
        let mut after = before.clone();
        let scope = &mut after
            .source
            .rust
            .budgets
            .as_mut()
            .expect("budgets")
            .overrides[0];
        match field {
            "target" => scope.budget.target += 1,
            "soft" => scope.budget.soft = Some(301),
            "hard" => scope.budget.hard += 1,
            _ => scope.budget.target_mode = SizeTargetMode::Warn,
        }
        assert!(
            changes(&before, &after)
                .iter()
                .any(|change| change.kind == ChangeKind::Grant),
            "{field}"
        );
    }
}

#[test]
fn exception_addition_expansion_redirection_and_removal_are_directional() {
    let before = configured();
    let mut after = before.clone();
    after
        .source
        .rust
        .budgets
        .as_mut()
        .expect("budgets")
        .exceptions[0]
        .hard += 1;
    assert!(
        changes(&before, &after)
            .iter()
            .any(|change| change.kind == ChangeKind::Grant)
    );
    after
        .source
        .rust
        .budgets
        .as_mut()
        .expect("budgets")
        .exceptions[0]
        .path = "src/new.rs".into();
    let delta = changes(&before, &after);
    assert!(
        delta.iter().any(
            |change| change.kind == ChangeKind::Grant && change.subject.ends_with("src/new.rs")
        )
    );
    assert!(
        delta
            .iter()
            .any(|change| change.kind == ChangeKind::Revoke
                && change.subject.ends_with("src/debt.rs"))
    );
    after
        .source
        .rust
        .budgets
        .as_mut()
        .expect("budgets")
        .exceptions
        .clear();
    assert!(
        changes(&before, &after)
            .iter()
            .all(|change| change.kind == ChangeKind::Revoke)
    );
}

#[test]
fn selector_changes_are_protected_but_toml_set_order_is_neutral() {
    let before = configured();
    let mut after = before.clone();
    after
        .source
        .rust
        .budgets
        .as_mut()
        .expect("budgets")
        .overrides[0]
        .roles
        .reverse();
    assert!(changes(&before, &after).is_empty());
    after
        .source
        .rust
        .budgets
        .as_mut()
        .expect("budgets")
        .overrides[0]
        .include = vec!["**".into()];
    assert!(
        changes(&before, &after)
            .iter()
            .any(|change| change.kind == ChangeKind::Unknown)
    );
}

#[test]
fn authored_baseline_cannot_be_removed_or_inflated_behind_an_unchanged_lock() {
    let before = configured();
    for baseline in [None, Some(531)] {
        let mut after = before.clone();
        after.ratchets[0].baseline = baseline;
        assert!(
            changes(&before, &after)
                .iter()
                .any(|change| change.kind == ChangeKind::Grant
                    && change.rail == "rust.file-size.baseline")
        );
    }
    let mut after = before.clone();
    after.ratchets[0].baseline = Some(529);
    assert!(
        changes(&before, &after)
            .iter()
            .all(|change| change.kind == ChangeKind::Revoke)
    );
}

#[test]
fn disabling_independent_hard_enforcement_is_a_grant() {
    let before = configured();
    let mut after = before.clone();
    after.source.rust.budgets = None;
    assert!(
        changes(&before, &after)
            .iter()
            .any(|change| change.kind == ChangeKind::Grant
                && change.subject == "source.rust.budgets")
    );
}

#[test]
fn removing_or_exchanging_required_accountability_fields_exposes_grants() {
    use crate::SizeExceptionMetadata;
    let mut before = configured();
    before
        .source
        .rust
        .budgets
        .as_mut()
        .expect("budgets")
        .exception_metadata = SizeExceptionMetadata::OwnerIssue;
    for metadata in [
        SizeExceptionMetadata::Tracking,
        SizeExceptionMetadata::OwnerIssueOrTracking,
    ] {
        let mut after = before.clone();
        after
            .source
            .rust
            .budgets
            .as_mut()
            .expect("budgets")
            .exception_metadata = metadata;
        assert!(
            changes(&before, &after)
                .iter()
                .any(|change| change.kind == ChangeKind::Grant
                    && change.subject.ends_with("exception-metadata"))
        );
    }
}

fn changes(before: &Contract, after: &Contract) -> Vec<crate::ArchitectureChange> {
    let mut changes = Vec::new();
    compare(before, after, &mut changes);
    changes
}
