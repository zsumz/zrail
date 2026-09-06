//! Literal implication is bounded to known modes; uncertain transformations stay protected.

use crate::{RepositoryCaseMode, RepositoryLiteralMode, RepositoryLiteralPredicate};

use super::{ChangeKind, interval};

pub(super) fn compare(
    left: &RepositoryLiteralPredicate,
    right: &RepositoryLiteralPredicate,
) -> Vec<ChangeKind> {
    use RepositoryLiteralMode::{Absent, Contains, EndsWith, Equals, ExactCount, StartsWith};
    if left.normalization != right.normalization {
        return vec![ChangeKind::Unknown];
    }
    if left.case != right.case {
        if left.text != right.text || left.mode != right.mode || left.count != right.count {
            return vec![ChangeKind::Unknown];
        }
        let insensitive = right.case == RepositoryCaseMode::AsciiInsensitive;
        return match (left.mode, left.count) {
            (Absent, _) | (ExactCount, Some(0)) => vec![if insensitive {
                ChangeKind::Revoke
            } else {
                ChangeKind::Grant
            }],
            (ExactCount, _) => vec![ChangeKind::Unknown],
            _ => vec![if insensitive {
                ChangeKind::Grant
            } else {
                ChangeKind::Revoke
            }],
        };
    }
    let fold = |text: &str| match left.case {
        RepositoryCaseMode::Sensitive => text.to_owned(),
        RepositoryCaseMode::AsciiInsensitive => text.to_ascii_lowercase(),
    };
    let (a, b) = (fold(&left.text), fold(&right.text));
    if a == b {
        if let (Some(a), Some(b)) = (counts(left), counts(right)) {
            return interval(a, b);
        }
        if left.mode == right.mode && left.count == right.count {
            return Vec::new();
        }
        return match (left.mode, right.mode) {
            (Equals, StartsWith | EndsWith | Contains) | (StartsWith | EndsWith, Contains) => {
                vec![ChangeKind::Grant]
            }
            (StartsWith | EndsWith | Contains, Equals) | (Contains, StartsWith | EndsWith) => {
                vec![ChangeKind::Revoke]
            }
            (Equals, ExactCount) if right.count == Some(1) => vec![ChangeKind::Grant],
            (ExactCount, Equals) if left.count == Some(1) => vec![ChangeKind::Revoke],
            _ => vec![ChangeKind::Unknown],
        };
    }
    if left.mode != right.mode || left.count != right.count {
        return vec![ChangeKind::Unknown];
    }
    let (new_implies_old, old_implies_new) = match left.mode {
        Contains => (b.contains(&a), a.contains(&b)),
        StartsWith => (b.starts_with(&a), a.starts_with(&b)),
        EndsWith => (b.ends_with(&a), a.ends_with(&b)),
        Absent => (a.contains(&b), b.contains(&a)),
        ExactCount if left.count == Some(0) => (a.contains(&b), b.contains(&a)),
        Equals => return vec![ChangeKind::Grant, ChangeKind::Revoke],
        ExactCount => return vec![ChangeKind::Unknown],
    };
    match (new_implies_old, old_implies_new) {
        (true, false) => vec![ChangeKind::Revoke],
        (false, true) => vec![ChangeKind::Grant],
        _ => vec![ChangeKind::Unknown],
    }
}

fn counts(literal: &RepositoryLiteralPredicate) -> Option<(usize, Option<usize>)> {
    match literal.mode {
        RepositoryLiteralMode::Contains => Some((1, None)),
        RepositoryLiteralMode::Absent => Some((0, Some(0))),
        RepositoryLiteralMode::ExactCount => literal.count.map(|count| (count, Some(count))),
        _ => None,
    }
}
