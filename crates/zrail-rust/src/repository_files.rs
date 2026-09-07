//! Closed repository-file enforcement shares bounded inventory and immutable content facts.

mod boundary;
mod diagnostic;
mod documents;
mod input;
mod literal;
mod model;
mod predicates;
mod select;
mod text_model;
mod text_structure;

use std::path::Path;

use zrail_core::{
    AnalysisQuality, RepositoryFilePredicate, RepositoryFileRule, RepositoryNameBasis, sha256_hex,
};

pub(crate) use model::RepositoryFileAnalysis;
pub use model::{
    GovernedRepositoryFile, GovernedRepositoryFileEntry, RepositoryDocumentKeys,
    RepositoryDocumentObservation,
};
pub(crate) use select::Selection;
pub use text_model::{RepositoryLineValue, RepositoryTextStructureObservation};

pub(crate) fn analyze(
    root: &Path,
    rules: &[RepositoryFileRule],
) -> Result<RepositoryFileAnalysis, String> {
    let mut analysis = RepositoryFileAnalysis::default();
    if rules.is_empty() {
        return Ok(analysis);
    }
    let mut selection = Selection::new(
        root,
        rules
            .iter()
            .flat_map(|rule| rule.include.iter().map(String::as_str)),
    )
    .map_err(incomplete)?;
    let mut inputs = input::Inputs::default();
    let mut rules = rules.iter().collect::<Vec<_>>();
    rules.sort_by(|left, right| left.name.cmp(&right.name));
    for rule in rules {
        let policy_id = format!("repository:file:{}", rule.name);
        let context_error = |message| incomplete(format!("{policy_id}: {message}"));
        let mut observed = GovernedRepositoryFile {
            policy_id: policy_id.clone(),
            policy: rule.clone(),
            claim: match rule.predicate {
                RepositoryFilePredicate::Literal(ref literal)
                    if matches!(
                        literal.mode,
                        zrail_core::RepositoryLiteralMode::LinePresent
                            | zrail_core::RepositoryLiteralMode::LineAbsent
                    ) =>
                {
                    "raw-utf8-lines"
                }
                RepositoryFilePredicate::Literal(_) => "raw-utf8-text",
                RepositoryFilePredicate::LiteralOrder { .. }
                | RepositoryFilePredicate::LiteralBetween { .. } => "raw-utf8-byte-interval",
                RepositoryFilePredicate::LineValuesAllowed { .. } => "raw-utf8-line-values",
                RepositoryFilePredicate::LinePrefixesAbsent { .. } => "raw-utf8-line-prefixes",
                RepositoryFilePredicate::Document(_) => "authored-document",
                RepositoryFilePredicate::BytesEqual { utf8: true, .. } => "utf8-file-bytes",
                RepositoryFilePredicate::BytesEqual { .. } => "file-bytes",
                _ => "physical-paths",
            }
            .into(),
            analysis: AnalysisQuality::Exact,
            checkout_path: if matches!(
                rule.predicate,
                RepositoryFilePredicate::ForbiddenNames {
                    basis: RepositoryNameBasis::Filesystem,
                    ..
                }
            ) {
                Some(
                    root.to_str()
                        .ok_or_else(|| incomplete("checkout path is not UTF-8"))?
                        .into(),
                )
            } else {
                None
            },
            entries: selection
                .select(root, &rule.include, &rule.exclude, rule.entry)
                .map_err(&context_error)?,
            reference: None,
            satisfied: false,
        };
        let diagnostic =
            predicates::evaluate(root, &mut observed, &mut inputs).map_err(context_error)?;
        if !observed.satisfied {
            analysis
                .findings
                .push(diagnostic::finding(&observed, diagnostic));
        }
        analysis.policies.push(observed);
    }
    let bytes =
        serde_json::to_vec(&analysis.policies).map_err(|error| incomplete(error.to_string()))?;
    analysis.binding_sha256 = Some(sha256_hex(&bytes));
    Ok(analysis)
}

fn incomplete(message: impl std::fmt::Display) -> String {
    format!("REP-FILE-006: incomplete repository-file analysis: {message}")
}

#[cfg(test)]
#[path = "repository_files_test.rs"]
mod repository_files_test;
