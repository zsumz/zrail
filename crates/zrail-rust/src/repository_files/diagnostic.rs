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
        RepositoryFilePredicate::BytesEqual { other, utf8 } => format!(
            "byte equality with {other:?} failed: reference present {}, UTF-8 required {utf8}, reference valid UTF-8 {:?}, {} selected files, {failed} unsatisfied files",
            observed.reference.is_some(),
            observed
                .reference
                .as_ref()
                .and_then(|entry| entry.valid_utf8),
            observed.entries.len(),
        ),
        RepositoryFilePredicate::Document(document) => format!(
            "authored {:?} document predicate {:?} at literal keys {:?} failed: {} selected files, {failed} unsatisfied files; first selection {:?}",
            document.format,
            document.assertion,
            document.path,
            observed.entries.len(),
            observed
                .entries
                .iter()
                .find(|entry| !entry.satisfied)
                .and_then(|entry| entry.document.as_ref()),
        ),
        RepositoryFilePredicate::LiteralOrder { .. }
        | RepositoryFilePredicate::LiteralBetween { .. }
        | RepositoryFilePredicate::LineValuesAllowed { .. }
        | RepositoryFilePredicate::LinePrefixesAbsent { .. } => format!(
            "raw {} predicate failed: {} selected files, {failed} unsatisfied files; first observation {:?}",
            match observed.policy.predicate {
                RepositoryFilePredicate::LiteralOrder { .. } => "first-marker order",
                RepositoryFilePredicate::LiteralBetween { .. } => "first-marker interval",
                RepositoryFilePredicate::LinePrefixesAbsent { .. } => "forbidden line-prefix set",
                _ => "prefixed line-value set",
            },
            observed.entries.len(),
            observed
                .entries
                .iter()
                .find(|entry| !entry.satisfied)
                .and_then(|entry| entry.text_structure.as_ref()),
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
