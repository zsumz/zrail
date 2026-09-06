//! Removing structured evidence and exchanging ordered subjects require protected review.

use super::{configured, kinds, protected};
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
