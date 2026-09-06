//! Protected review accounts for counts, exact identities, quantified scopes, and raw modes.

#[path = "files/documents_test.rs"]
mod documents_test;

use crate::diff::compare_fixture_test::contract_with_hard_limit;
use crate::{ChangeKind, Contract, RepositoryEntryMode, RepositoryFileRule, compare_architecture};

fn configured(predicate: &str) -> Contract {
    let mut contract = contract_with_hard_limit(500);
    contract.repository.files.push(
        toml::from_str::<RepositoryFileRule>(&format!(
            r#"
name = "fixture"
include = ["data/**"]
reason = "Reviewed structural requirement."
predicate = {{ {predicate} }}
"#
        ))
        .expect("file policy"),
    );
    contract
}

fn kinds(before: &Contract, after: &Contract) -> Vec<ChangeKind> {
    let report = compare_architecture(before, None, after, None);
    let mut kinds = report
        .changes
        .iter()
        .filter(|change| change.rail == "repository.files")
        .map(|change| {
            assert_eq!(change.subject, "repository:file:fixture");
            change.kind
        })
        .collect::<Vec<_>>();
    kinds.sort();
    kinds.dedup();
    kinds
}

fn protected(before: &Contract, after: &Contract) {
    assert!(compare_architecture(before, None, after, None).denies_grants());
}

#[test]
fn count_bounds_compare_accepted_intervals_and_removal_is_a_grant() {
    let before = configured("kind = 'count', minimum = 2, maximum = 4");
    for predicate in [
        "kind = 'count', minimum = 1, maximum = 4",
        "kind = 'count', minimum = 2, maximum = 5",
        "kind = 'count', minimum = 2",
    ] {
        let after = configured(predicate);
        assert_eq!(kinds(&before, &after), [ChangeKind::Grant]);
        protected(&before, &after);
    }
    let mut after = before.clone();
    after.repository.files.clear();
    assert_eq!(kinds(&before, &after), [ChangeKind::Grant]);
    assert_eq!(kinds(&after, &before), [ChangeKind::Revoke]);
    protected(&before, &after);
}

#[test]
fn exact_counts_do_not_treat_a_smaller_number_as_tightening() {
    let before = configured("kind = 'literal', text = 'step', mode = 'exact-count', count = 3");
    let after = configured("kind = 'literal', text = 'step', mode = 'exact-count', count = 2");
    assert_eq!(
        kinds(&before, &after),
        [ChangeKind::Grant, ChangeKind::Revoke]
    );
    protected(&before, &after);
    let before = configured("kind = 'literal', text = 'step', mode = 'absent'");
    let after = configured("kind = 'literal', text = 'step', mode = 'exact-count', count = 0");
    assert!(kinds(&before, &after).is_empty());
}

#[test]
fn exact_sets_ignore_order_but_never_allow_identity_changes_without_review() {
    let before = configured("kind = 'exact-paths', paths = ['data/a', 'data/b']");
    let reordered = configured("kind = 'exact-paths', paths = ['data/b', 'data/a']");
    assert!(kinds(&before, &reordered).is_empty());
    for paths in [
        "['data/a']",
        "['data/a', 'data/c']",
        "['data/a', 'data/b', 'data/c']",
    ] {
        let after = configured(&format!("kind = 'exact-paths', paths = {paths}"));
        assert_eq!(
            kinds(&before, &after),
            [ChangeKind::Grant, ChangeKind::Revoke]
        );
        protected(&before, &after);
    }
    let mut after = before.clone();
    after.repository.files[0].entry = RepositoryEntryMode::Any;
    protected(&before, &after);
}

#[test]
fn broader_presence_selection_is_a_grant_but_broader_prohibition_tightens() {
    for (predicate, expected) in [
        ("kind = 'count', minimum = 1", ChangeKind::Grant),
        (
            "kind = 'count', minimum = 0, maximum = 0",
            ChangeKind::Revoke,
        ),
        (
            "kind = 'literal', text = 'bad', mode = 'absent'",
            ChangeKind::Revoke,
        ),
        (
            "kind = 'literal', text = 'needed', mode = 'contains'",
            ChangeKind::Unknown,
        ),
    ] {
        let before = configured(predicate);
        let mut after = before.clone();
        after.repository.files[0].include.push("docs/**".into());
        assert_eq!(kinds(&before, &after), [expected]);
        if expected != ChangeKind::Revoke {
            protected(&before, &after);
        }
    }
}

#[test]
fn expanded_prohibition_exclusions_and_changed_globs_cannot_be_auto_accepted() {
    let before = configured("kind = 'literal', text = 'bad', mode = 'absent'");
    let mut after = before.clone();
    after.repository.files[0]
        .exclude
        .push("data/escape/**".into());
    assert_eq!(kinds(&before, &after), [ChangeKind::Grant]);
    protected(&before, &after);
    after.repository.files[0].include = vec!["data/*.rs".into()];
    assert_eq!(kinds(&before, &after), [ChangeKind::Unknown]);
    protected(&before, &after);
}

#[test]
fn weakened_required_literals_and_weakened_forbidden_literals_are_grants() {
    for (before, after) in [
        (
            "text = 'required step', mode = 'contains'",
            "text = 'step', mode = 'contains'",
        ),
        (
            "text = 'bad', mode = 'absent'",
            "text = 'bad suffix', mode = 'absent'",
        ),
        (
            "text = 'header', mode = 'starts-with'",
            "text = 'header', mode = 'contains'",
        ),
        (
            "text = 'header', mode = 'equals'",
            "text = 'header', mode = 'ends-with'",
        ),
        (
            "text = 'Header', mode = 'equals'",
            "text = 'Header', mode = 'equals', case = 'ascii-insensitive'",
        ),
        (
            "text = 'bad', mode = 'absent', case = 'ascii-insensitive'",
            "text = 'bad', mode = 'absent'",
        ),
    ] {
        let before = configured(&format!("kind = 'literal', {before}"));
        let after = configured(&format!("kind = 'literal', {after}"));
        assert_eq!(kinds(&before, &after), [ChangeKind::Grant]);
        protected(&before, &after);
    }
}

#[test]
fn uncertain_transformations_and_changed_byte_references_require_review() {
    let before = configured("kind = 'literal', text = 'step', mode = 'contains'");
    let after = configured(
        "kind = 'literal', text = 'step', mode = 'contains', normalization = 'remove-whitespace'",
    );
    assert_eq!(kinds(&before, &after), [ChangeKind::Unknown]);
    protected(&before, &after);
    let before = configured("kind = 'bytes-equal', other = 'LICENSE'");
    let after = configured("kind = 'bytes-equal', other = 'OTHER-LICENSE'");
    assert_eq!(
        kinds(&before, &after),
        [ChangeKind::Grant, ChangeKind::Revoke]
    );
    protected(&before, &after);
}

#[test]
fn dropping_utf8_equality_is_a_grant_and_defaults_retain_binary_authority() {
    let binary = configured("kind = 'bytes-equal', other = 'LICENSE'");
    let explicit = configured("kind = 'bytes-equal', other = 'LICENSE', utf8 = false");
    assert_eq!(binary, explicit);
    assert!(
        !toml::to_string(&binary.repository.files[0])
            .expect("serialize")
            .contains("utf8")
    );
    let text = configured("kind = 'bytes-equal', other = 'LICENSE', utf8 = true");
    assert_eq!(kinds(&binary, &text), [ChangeKind::Revoke]);
    assert_eq!(kinds(&text, &binary), [ChangeKind::Grant]);
    protected(&text, &binary);
}

#[test]
fn name_checks_preserve_each_authorized_component_case_and_checkout_boundary() {
    let before = configured(
        "kind = 'forbidden-names', names = ['bad', 'vague'], part = 'component-stem', case = 'ascii-insensitive', basis = 'filesystem'",
    );
    for predicate in [
        "names = ['bad'], part = 'component-stem', case = 'ascii-insensitive', basis = 'filesystem'",
        "names = ['bad', 'vague'], part = 'file-stem', case = 'ascii-insensitive', basis = 'filesystem'",
        "names = ['bad', 'vague'], part = 'component-stem', case = 'sensitive', basis = 'filesystem'",
        "names = ['bad', 'vague'], part = 'component-stem', case = 'ascii-insensitive', basis = 'repository'",
    ] {
        let after = configured(&format!("kind = 'forbidden-names', {predicate}"));
        assert_eq!(kinds(&before, &after), [ChangeKind::Grant]);
        protected(&before, &after);
    }
}

#[test]
fn selector_reordering_is_neutral_and_justification_changes_remain_reviewed() {
    let mut before = configured("kind = 'count', minimum = 0, maximum = 0");
    before.repository.files[0].include.push("docs/**".into());
    let mut after = before.clone();
    after.repository.files[0].include.reverse();
    assert!(kinds(&before, &after).is_empty());
    after.repository.files[0].reason = "Different authority statement.".into();
    assert_eq!(kinds(&before, &after), [ChangeKind::Unknown]);
    protected(&before, &after);
}
