//! Proven selector-set inclusion is interpreted according to the predicate's quantifier.

use crate::{
    RepositoryEntryMode, RepositoryFilePredicate, RepositoryFileRule, RepositoryLiteralMode,
};

use super::{ChangeKind, set};

pub(super) fn compare(left: &RepositoryFileRule, right: &RepositoryFileRule) -> Vec<ChangeKind> {
    let left_include = set(&left.include);
    let right_include = set(&right.include);
    let left_exclude = set(&left.exclude);
    let right_exclude = set(&right.exclude);
    if left_include == right_include && left_exclude == right_exclude && left.entry == right.entry {
        return Vec::new();
    }
    if left.predicate != right.predicate {
        return vec![ChangeKind::Unknown];
    }
    if left.entry != right.entry
        && matches!(left.predicate, RepositoryFilePredicate::ExactPaths { .. })
    {
        return vec![ChangeKind::Unknown];
    }
    let expanded = left_include.is_subset(&right_include)
        && right_exclude.is_subset(&left_exclude)
        && kind_subset(left.entry, right.entry);
    let narrowed = right_include.is_subset(&left_include)
        && left_exclude.is_subset(&right_exclude)
        && kind_subset(right.entry, left.entry);
    if !expanded && !narrowed {
        return vec![ChangeKind::Unknown];
    }
    let expansion_tightens = match &left.predicate {
        RepositoryFilePredicate::Count { minimum: 0, .. }
        | RepositoryFilePredicate::ExactPaths { .. }
        | RepositoryFilePredicate::ForbiddenNames { .. } => Some(true),
        RepositoryFilePredicate::Document(document)
            if document.assertion == crate::RepositoryDocumentAssertion::Absent =>
        {
            Some(true)
        }
        RepositoryFilePredicate::Count { maximum: None, .. } => Some(false),
        RepositoryFilePredicate::Literal(literal)
            if matches!(
                literal.mode,
                RepositoryLiteralMode::Absent | RepositoryLiteralMode::LineAbsent
            ) || (literal.mode == RepositoryLiteralMode::ExactCount
                && literal.count == Some(0)) =>
        {
            Some(true)
        }
        _ => None,
    };
    match expansion_tightens {
        Some(tightens) => vec![if expanded == tightens {
            ChangeKind::Revoke
        } else {
            ChangeKind::Grant
        }],
        // Positive per-file predicates also require a nonempty selection. Scope
        // changes can weaken either the universal condition or that presence guard.
        None => vec![ChangeKind::Unknown],
    }
}

fn kind_subset(left: RepositoryEntryMode, right: RepositoryEntryMode) -> bool {
    left == right || right == RepositoryEntryMode::Any
}
