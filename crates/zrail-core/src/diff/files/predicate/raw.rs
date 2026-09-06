//! Raw predicate comparisons prove accepted-set inclusion or retain protected uncertainty.

use crate::{
    RepositoryCaseMode, RepositoryFilePredicate as Predicate, RepositoryLiteralMode,
    RepositoryTextNormalization,
};

use super::super::{ChangeKind, set};

pub(super) fn compare(left: &Predicate, right: &Predicate) -> Option<Vec<ChangeKind>> {
    match (left, right) {
        (
            Predicate::LineValuesAllowed {
                prefix: a,
                values: av,
                normalization: an,
            },
            Predicate::LineValuesAllowed {
                prefix: b,
                values: bv,
                normalization: bn,
            },
        ) if a == b && an == bn => {
            let mut changes = Vec::new();
            if !set(bv).is_subset(&set(av)) {
                changes.push(ChangeKind::Grant);
            }
            if !set(av).is_subset(&set(bv)) {
                changes.push(ChangeKind::Revoke);
            }
            Some(changes)
        }
        (
            Predicate::LiteralBetween {
                start: a,
                end: b,
                contains: av,
            },
            Predicate::LiteralBetween {
                start: c,
                end: d,
                contains: bv,
            },
        ) if a == c && b == d => Some(if av.contains(bv) {
            vec![ChangeKind::Grant]
        } else if bv.contains(av) {
            vec![ChangeKind::Revoke]
        } else {
            // Markers can themselves entail the searched text; arbitrary substitutions
            // cannot be assigned a permission direction from their spelling alone.
            vec![ChangeKind::Unknown]
        }),
        (structured, Predicate::Literal(literal)) if entails_literal(structured, literal) => {
            Some(vec![ChangeKind::Grant])
        }
        (Predicate::Literal(literal), structured) if entails_literal(structured, literal) => {
            Some(vec![ChangeKind::Revoke])
        }
        _ if is_structure(left) || is_structure(right) => Some(vec![ChangeKind::Unknown]),
        _ => None,
    }
}

fn entails_literal(structured: &Predicate, literal: &crate::RepositoryLiteralPredicate) -> bool {
    if literal.mode != RepositoryLiteralMode::Contains
        || literal.normalization != RepositoryTextNormalization::None
        || literal.case != RepositoryCaseMode::Sensitive
    {
        return false;
    }
    match structured {
        Predicate::LiteralOrder { before, after } => {
            literal.text == *before || literal.text == *after
        }
        Predicate::LiteralBetween {
            start,
            end,
            contains,
        } => [&**start, &**end, &**contains].contains(&literal.text.as_str()),
        _ => false,
    }
}

fn is_structure(predicate: &Predicate) -> bool {
    matches!(
        predicate,
        Predicate::LiteralOrder { .. }
            | Predicate::LiteralBetween { .. }
            | Predicate::LineValuesAllowed { .. }
    )
}

#[cfg(test)]
#[path = "raw_test.rs"]
mod raw_test;
