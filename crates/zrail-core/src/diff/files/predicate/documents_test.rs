//! Removing structured evidence and exchanging ordered subjects require protected review.

use crate::diff::files::files_test::{configured, kinds, protected};
use crate::{ChangeKind, Contract};

fn document(assertion: &str) -> Contract {
    configured(&format!(
        "kind = 'document', format = 'toml', path = ['package', 'publish'], assertion = {{ {assertion} }}"
    ))
}

#[test]
fn changed_exact_document_values_and_array_order_are_not_budget_tightening() {
    for (left, right) in [
        ("value = 2", "value = 1"),
        ("value = true", "value = false"),
        ("value = 'true'", "value = true"),
        ("value = ['a', 'b']", "value = ['b', 'a']"),
        ("value = ['a', 'b']", "value = ['a']"),
        ("value = ['a']", "value = ['a', 'a']"),
    ] {
        let left = document(&format!("op = 'equals', {left}"));
        let right = document(&format!("op = 'equals', {right}"));
        assert_eq!(
            kinds(&left, &right),
            [ChangeKind::Grant, ChangeKind::Revoke]
        );
        protected(&left, &right);
    }
}

#[test]
fn weakened_document_presence_selection_and_removal_remain_protected() {
    for stronger in ["op = 'equals', value = ['a']", "op = 'nonempty-string'"] {
        let before = document(stronger);
        let after = document("op = 'present'");
        assert_eq!(kinds(&before, &after), [ChangeKind::Grant]);
        assert_eq!(kinds(&after, &before), [ChangeKind::Revoke]);
        protected(&before, &after);
    }
    let before = document("op = 'absent'");
    let mut after = before.clone();
    after.repository.files[0]
        .exclude
        .push("data/escape/**".into());
    assert_eq!(kinds(&before, &after), [ChangeKind::Grant]);
    protected(&before, &after);
    after.repository.files.clear();
    assert_eq!(kinds(&before, &after), [ChangeKind::Grant]);
    protected(&before, &after);
    let redirected = configured(
        "kind = 'document', format = 'json', path = ['elsewhere'], assertion = { op = 'absent' }",
    );
    assert_eq!(kinds(&before, &redirected), [ChangeKind::Unknown]);
    protected(&before, &redirected);
    let exact = document("op = 'equals', value = 'reviewed metadata'");
    let nonempty = document("op = 'nonempty-string'");
    assert_eq!(kinds(&exact, &nonempty), [ChangeKind::Grant]);
    assert_eq!(kinds(&nonempty, &exact), [ChangeKind::Revoke]);
    protected(&exact, &nonempty);
}

#[test]
fn document_key_sets_compare_accepted_sets_and_explicit_type_permissions() {
    for (left, right, expected) in [
        (
            "keys-exact', keys = ['a', 'b']",
            "keys-exact', keys = ['b', 'a']",
            vec![],
        ),
        (
            "keys-exact', keys = ['a']",
            "keys-exact', keys = ['b']",
            vec![ChangeKind::Grant, ChangeKind::Revoke],
        ),
        (
            "keys-allowed', keys = ['a']",
            "keys-allowed', keys = ['a', 'b']",
            vec![ChangeKind::Grant],
        ),
        (
            "keys-exact', keys = ['a']",
            "keys-allowed', keys = ['a', 'b']",
            vec![ChangeKind::Grant],
        ),
        (
            "keys-allowed', keys = ['a', 'b']",
            "keys-exact', keys = ['a']",
            vec![ChangeKind::Revoke],
        ),
        ("keys-allowed', keys = []", "keys-exact', keys = []", vec![]),
        (
            "keys-allowed', keys = ['a']",
            "keys-exact', keys = ['b']",
            vec![ChangeKind::Grant, ChangeKind::Revoke],
        ),
        (
            "keys-exact', keys = ['a']",
            "keys-exact', keys = ['a'], empty_on_non_table = true",
            vec![],
        ),
        (
            "keys-exact', keys = []",
            "keys-exact', keys = [], empty_on_non_table = true",
            vec![ChangeKind::Grant],
        ),
        (
            "keys-allowed', keys = ['a']",
            "keys-allowed', keys = ['a'], empty_on_non_table = true",
            vec![ChangeKind::Grant],
        ),
    ] {
        let before = document(&format!("op = '{left}"));
        let after = document(&format!("op = '{right}"));
        assert_eq!(kinds(&before, &after), expected, "{left} -> {right}");
        let mut reverse = expected
            .iter()
            .map(|kind| match kind {
                ChangeKind::Grant => ChangeKind::Revoke,
                ChangeKind::Revoke => ChangeKind::Grant,
                _ => panic!("unexpected comparison kind"),
            })
            .collect::<Vec<_>>();
        reverse.sort();
        assert_eq!(kinds(&after, &before), reverse);
        if expected.contains(&ChangeKind::Grant) {
            protected(&before, &after);
        }
    }
    for op in ["keys-exact", "keys-allowed"] {
        let before = document(&format!("op = '{op}', keys = ['a']"));
        let after = document("op = 'present'");
        assert_eq!(kinds(&before, &after), [ChangeKind::Grant]);
        assert_eq!(kinds(&after, &before), [ChangeKind::Revoke]);
        protected(&before, &after);
    }
}

#[test]
fn removing_or_redirecting_a_typed_field_prohibition_stays_protected() {
    let before = document("op = 'field-not-string', field = 'version'");
    let after = document("op = 'present'");
    assert_eq!(kinds(&before, &after), [ChangeKind::Grant]);
    assert_eq!(kinds(&after, &before), [ChangeKind::Revoke]);
    protected(&before, &after);
    let redirected = document("op = 'field-not-string', field = 'elsewhere'");
    assert_eq!(kinds(&before, &redirected), [ChangeKind::Unknown]);
    protected(&before, &redirected);
}
