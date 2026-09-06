//! Exact sets, count intervals, and name prohibitions have distinct permission directions.

use crate::{RepositoryCaseMode, RepositoryFilePredicate, RepositoryNameBasis, RepositoryNamePart};

use super::{ChangeKind, interval, literal, set};

mod documents;

pub(super) fn compare(
    left: &RepositoryFilePredicate,
    right: &RepositoryFilePredicate,
) -> Vec<ChangeKind> {
    use RepositoryFilePredicate::{BytesEqual, Count, ExactPaths, ForbiddenNames, Literal};
    if left == right {
        return Vec::new();
    }
    match (left, right) {
        (
            Count {
                minimum: a,
                maximum: b,
            },
            Count {
                minimum: c,
                maximum: d,
            },
        ) => interval((*a, *b), (*c, *d)),
        (ExactPaths { paths: a }, ExactPaths { paths: b }) if set(a) == set(b) => Vec::new(),
        (BytesEqual { other: a, utf8: au }, BytesEqual { other: b, utf8: bu }) if a == b => {
            match (au, bu) {
                (true, false) => vec![ChangeKind::Grant],
                (false, true) => vec![ChangeKind::Revoke],
                _ => Vec::new(),
            }
        }
        (ExactPaths { .. }, ExactPaths { .. }) | (BytesEqual { .. }, BytesEqual { .. }) => {
            // Changing an exact subject both allows a previously disallowed state
            // and rejects a formerly accepted one. Neither direction subsumes it.
            vec![ChangeKind::Grant, ChangeKind::Revoke]
        }
        (Literal(a), Literal(b)) => literal::compare(a, b),
        (RepositoryFilePredicate::Document(a), RepositoryFilePredicate::Document(b)) => {
            documents::compare(a, b)
        }
        (
            ForbiddenNames {
                names: a,
                part: ap,
                basis: ab,
                case: ac,
            },
            ForbiddenNames {
                names: b,
                part: bp,
                basis: bb,
                case: bc,
            },
        ) => {
            let mut kinds = Vec::new();
            let a = names(a, *ac);
            let b = names(b, *ac);
            if !set(&a).is_subset(&set(&b)) {
                kinds.push(ChangeKind::Grant);
            }
            if !set(&b).is_subset(&set(&a)) {
                kinds.push(ChangeKind::Revoke);
            }
            if ac != bc {
                kinds.push(if *bc == RepositoryCaseMode::AsciiInsensitive {
                    ChangeKind::Revoke
                } else {
                    ChangeKind::Grant
                });
            }
            if ap != bp {
                kinds.push(match (ap, bp) {
                    (RepositoryNamePart::FileName, RepositoryNamePart::Component)
                    | (RepositoryNamePart::FileStem, RepositoryNamePart::ComponentStem) => {
                        ChangeKind::Revoke
                    }
                    (RepositoryNamePart::Component, RepositoryNamePart::FileName)
                    | (RepositoryNamePart::ComponentStem, RepositoryNamePart::FileStem) => {
                        ChangeKind::Grant
                    }
                    _ => ChangeKind::Unknown,
                });
            }
            if ab != bb
                && matches!(
                    bp,
                    RepositoryNamePart::Component | RepositoryNamePart::ComponentStem
                )
            {
                kinds.push(if *bb == RepositoryNameBasis::Repository {
                    ChangeKind::Grant
                } else {
                    ChangeKind::Revoke
                });
            }
            kinds.sort();
            kinds.dedup();
            kinds
        }
        _ => vec![ChangeKind::Unknown],
    }
}

fn names(values: &[String], case: RepositoryCaseMode) -> Vec<String> {
    values
        .iter()
        .map(|value| match case {
            RepositoryCaseMode::Sensitive => value.clone(),
            RepositoryCaseMode::AsciiInsensitive => value.to_ascii_lowercase(),
        })
        .collect()
}
