//! Exact document values and ordered arrays are authority subjects, not numerical budgets.

#[cfg(test)]
#[path = "documents_test.rs"]
mod documents_test;

use crate::{
    ChangeKind, RepositoryDocumentAssertion as Assertion, RepositoryDocumentPredicate,
    RepositoryDocumentValue,
};
use std::collections::BTreeSet;

pub(super) fn compare(
    left: &RepositoryDocumentPredicate,
    right: &RepositoryDocumentPredicate,
) -> Vec<ChangeKind> {
    if left == right {
        return Vec::new();
    }
    if left.format != right.format || left.path != right.path {
        return vec![ChangeKind::Unknown];
    }
    if let (Some(left), Some(right)) = (key_set(&left.assertion), key_set(&right.assertion)) {
        let mut changes = Vec::new();
        if !right.implies(&left) {
            changes.push(ChangeKind::Grant);
        }
        if !left.implies(&right) {
            changes.push(ChangeKind::Revoke);
        }
        return changes;
    }
    match (&left.assertion, &right.assertion) {
        (
            Assertion::Equals { .. }
            | Assertion::NonemptyString
            | Assertion::KeysExact { .. }
            | Assertion::KeysAllowed { .. },
            Assertion::Present,
        ) => vec![ChangeKind::Grant],
        (
            Assertion::Present,
            Assertion::Equals { .. }
            | Assertion::NonemptyString
            | Assertion::KeysExact { .. }
            | Assertion::KeysAllowed { .. },
        ) => vec![ChangeKind::Revoke],
        (
            Assertion::Equals {
                value: RepositoryDocumentValue::String(value),
            },
            Assertion::NonemptyString,
        ) if !value.trim().is_empty() => vec![ChangeKind::Grant],
        (
            Assertion::NonemptyString,
            Assertion::Equals {
                value: RepositoryDocumentValue::String(value),
            },
        ) if !value.trim().is_empty() => vec![ChangeKind::Revoke],
        // Changed exact values, order, or presence polarity may admit old failures
        // and reject old successes. Unknown mixed predicates also stay protected.
        (Assertion::Equals { .. }, Assertion::Equals { .. })
        | (Assertion::Equals { .. }, Assertion::NonemptyString)
        | (Assertion::NonemptyString, Assertion::Equals { .. })
        | (Assertion::Absent, _)
        | (_, Assertion::Absent) => vec![ChangeKind::Grant, ChangeKind::Revoke],
        _ => vec![ChangeKind::Unknown],
    }
}

struct KeySet<'a> {
    keys: BTreeSet<&'a str>,
    exact: bool,
    accepts_non_table: bool,
}

impl KeySet<'_> {
    fn implies(&self, other: &Self) -> bool {
        if self.accepts_non_table && !other.accepts_non_table {
            return false;
        }
        match (self.exact, other.exact) {
            (true, true) => self.keys == other.keys,
            (true, false) | (false, false) => self.keys.is_subset(&other.keys),
            // An empty allowlist accepts only the empty table, just like exact [].
            (false, true) => self.keys.is_empty() && other.keys.is_empty(),
        }
    }
}

fn key_set(assertion: &Assertion) -> Option<KeySet<'_>> {
    let (keys, empty_on_non_table, exact) = match assertion {
        Assertion::KeysExact {
            keys,
            empty_on_non_table,
        } => (keys, empty_on_non_table, true),
        Assertion::KeysAllowed {
            keys,
            empty_on_non_table,
        } => (keys, empty_on_non_table, false),
        _ => return None,
    };
    Some(KeySet {
        keys: keys.iter().map(String::as_str).collect(),
        exact,
        accepts_non_table: *empty_on_non_table && (!exact || keys.is_empty()),
    })
}
