//! File findings report exact failing quantities and bounded, actionable path examples.

use std::collections::BTreeSet;

use zrail_core::{Finding, RepositoryFilePredicate};

use super::model::GovernedRepositoryFile;

pub(super) fn finding(observed: &GovernedRepositoryFile, diagnostic: &str) -> Finding {
    let failed = observed
        .entries
        .iter()
        .filter(|entry| !entry.satisfied)
        .count();
    let message = match &observed.policy.predicate {
        RepositoryFilePredicate::Count { minimum, maximum } => format!(
            "selected {} entries; required minimum {minimum}, maximum {}",
            observed.entries.len(),
            maximum.map_or_else(|| "unbounded".into(), |value| value.to_string()),
        ),
        RepositoryFilePredicate::ExactPaths { paths } => {
            let expected = paths.iter().map(String::as_str).collect::<BTreeSet<_>>();
            let actual = observed
                .entries
                .iter()
                .map(|entry| entry.path.as_str())
                .collect::<BTreeSet<_>>();
            let missing = expected.difference(&actual).copied().collect::<Vec<_>>();
            let extra = actual.difference(&expected).copied().collect::<Vec<_>>();
            format!(
                "exact inventory has {} missing and {} unexpected paths; first missing {:?}; first unexpected {:?}",
                missing.len(),
                extra.len(),
                &missing[..missing.len().min(8)],
                &extra[..extra.len().min(8)]
            )
        }
        RepositoryFilePredicate::ForbiddenNames {
            part, basis, case, ..
        } => format!(
            "{failed} selected paths contain prohibited {part:?} names ({basis:?}, {case:?}); first names {:?}",
            observed
                .entries
                .iter()
                .find(|entry| !entry.satisfied)
                .map(|entry| &entry.forbidden_names),
        ),
        RepositoryFilePredicate::Literal(literal) if observed.entries.is_empty() => format!(
            "required {:?} raw-text predicate has no selected file; include {:?}, exclude {:?}",
            literal.mode, observed.policy.include, observed.policy.exclude,
        ),
        RepositoryFilePredicate::Literal(literal) => format!(
            "{failed} files violate raw {:?} text predicate; expected count {:?}, first observed count {:?}; normalization {:?}, case {:?}",
            literal.mode,
            literal.count,
            observed
                .entries
                .iter()
                .find(|entry| !entry.satisfied)
                .and_then(|entry| entry.literal_count),
            literal.normalization,
            literal.case,
        ),
        RepositoryFilePredicate::BytesEqual { other } => format!(
            "byte equality with {other:?} failed: reference present {}, {} selected files, {failed} unequal files",
            observed.reference.is_some(),
            observed.entries.len(),
        ),
    };
    let mut finding = Finding::error(diagnostic, &observed.policy_id, "repository-files", message)
        .because(&observed.policy.reason)
        .with_help("restore the required file contract; coverage lists the full selector, predicate, paths, hashes, and untruncated quantities");
    if let Some(entry) = observed.entries.iter().find(|entry| !entry.satisfied) {
        finding = finding.at(&entry.path, None);
    }
    finding
}
