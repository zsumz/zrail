//! Exact document values and ordered arrays are authority subjects, not numerical budgets.

use crate::{
    ChangeKind, RepositoryDocumentAssertion as Assertion, RepositoryDocumentPredicate,
    RepositoryDocumentValue,
};

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
    match (&left.assertion, &right.assertion) {
        (Assertion::Equals { .. } | Assertion::NonemptyString, Assertion::Present) => {
            vec![ChangeKind::Grant]
        }
        (Assertion::Present, Assertion::Equals { .. } | Assertion::NonemptyString) => {
            vec![ChangeKind::Revoke]
        }
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
