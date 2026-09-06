//! Strict bounded validation for repository-file assertions and literal semantics.

mod content;

use std::collections::BTreeSet;

use super::{
    Contract, RepositoryEntryMode, RepositoryFilePredicate, RepositoryLiteralMode,
    RepositoryTextNormalization,
    validate_limits::ValidationErrors,
    validate_paths::{validate_repository_literal, validate_repository_pattern},
    validate_sets::require_reason,
};
use crate::{RepositoryDocumentAssertion, RepositoryDocumentValue};

pub(super) fn validate(contract: &Contract, errors: &mut ValidationErrors) {
    let rules = &contract.repository.files;
    if rules.len() > 1_024 {
        errors.push("repository.files exceeds the 1024-rule safety limit".into());
    }
    let mut names = BTreeSet::new();
    for rule in rules {
        if rule.name.is_empty() || rule.name.trim() != rule.name || !names.insert(&rule.name) {
            errors.push("repository file assertions require unique nonempty names".into());
        }
        require_reason(
            "repository file assertion",
            &rule.name,
            &rule.reason,
            errors,
        );
        if rule.include.is_empty() || rule.include.len() + rule.exclude.len() > 64 {
            errors.push(format!(
                "file assertion {:?} requires 1..64 selectors",
                rule.name
            ));
        }
        for patterns in [&rule.include, &rule.exclude] {
            unique(patterns, "file assertion selectors", errors);
            for pattern in patterns {
                validate_repository_pattern(pattern, errors);
                exact(pattern, errors);
                if pattern.contains('\0') || pattern.split('/').any(str::is_empty) {
                    errors.push(format!(
                        "file assertion path pattern is not canonical: {pattern:?}"
                    ));
                }
            }
        }
        match &rule.predicate {
            RepositoryFilePredicate::Count { minimum, maximum } => {
                if maximum.is_some_and(|maximum| maximum < *minimum)
                    || (*minimum == 0 && maximum.is_none())
                {
                    errors.push(format!(
                        "file assertion {:?} has an empty or vacuous count constraint",
                        rule.name
                    ));
                }
            }
            RepositoryFilePredicate::ExactPaths { paths } => {
                if paths.len() > 20_000 {
                    errors.push("exact file inventory exceeds 20000 paths".into());
                }
                unique(paths, "exact file inventory", errors);
                for path in paths {
                    validate_repository_literal(path, errors);
                    exact(path, errors);
                    if !rule
                        .include
                        .iter()
                        .any(|pattern| crate::glob_matches(pattern, path))
                        || rule
                            .exclude
                            .iter()
                            .any(|pattern| crate::glob_matches(pattern, path))
                    {
                        errors.push(format!(
                            "exact file inventory path {path:?} falls outside its selection"
                        ));
                    }
                }
            }
            RepositoryFilePredicate::ForbiddenNames { names, .. } => {
                if names.is_empty() || names.len() > 256 {
                    errors.push("forbidden-name assertions require 1..256 names".into());
                }
                unique(names, "forbidden names", errors);
                for name in names {
                    if name.is_empty() || name.len() > 255 || name.contains(['/', '\\', '\0']) {
                        errors.push(format!("invalid literal path name {name:?}"));
                    }
                }
            }
            RepositoryFilePredicate::Literal(literal) => {
                regular(rule.entry, &rule.name, errors);
                if literal.text.is_empty() || literal.text.len() > 16 * 1024 {
                    errors.push("file literals require 1..16384 UTF-8 bytes".into());
                }
                if (literal.mode == RepositoryLiteralMode::ExactCount) != literal.count.is_some() {
                    errors.push("file literal count is required only in exact-count mode".into());
                }
                if matches!(
                    literal.mode,
                    RepositoryLiteralMode::LinePresent | RepositoryLiteralMode::LineAbsent
                ) && literal.text.contains(['\n', '\r'])
                {
                    errors.push("line literals cannot contain CR or LF characters".into());
                }
                if matches!(
                    literal.mode,
                    RepositoryLiteralMode::LinePresent | RepositoryLiteralMode::LineAbsent
                ) && match literal.normalization {
                    RepositoryTextNormalization::TrimStart => {
                        literal.text.trim_start() != literal.text
                    }
                    RepositoryTextNormalization::Trim => literal.text.trim() != literal.text,
                    _ => false,
                } {
                    errors.push(
                        "line literals must already satisfy their whitespace normalization".into(),
                    );
                }
                if literal.normalization == RepositoryTextNormalization::RemoveWhitespace
                    && literal.text.chars().any(char::is_whitespace)
                {
                    errors.push(
                        "whitespace-removed literals must not themselves contain whitespace".into(),
                    );
                }
            }
            RepositoryFilePredicate::BytesEqual { other, .. } => {
                regular(rule.entry, &rule.name, errors);
                validate_repository_literal(other, errors);
                exact(other, errors);
                if other == "." {
                    errors.push(
                        "byte equality requires a file reference, not the root directory".into(),
                    );
                }
            }
            RepositoryFilePredicate::Document(_)
            | RepositoryFilePredicate::LiteralOrder { .. }
            | RepositoryFilePredicate::LiteralBetween { .. }
            | RepositoryFilePredicate::LineValuesAllowed { .. } => {
                regular(rule.entry, &rule.name, errors);
                content::validate(&rule.predicate, errors);
            }
        }
    }
}

fn exact(path: &str, errors: &mut ValidationErrors) {
    if path.contains('\0')
        || crate::normalize_relative(std::path::Path::new(path))
            .is_ok_and(|normalized| normalized != path)
    {
        errors.push(format!("file assertion path is not canonical: {path:?}"));
    }
}

fn regular(entry: RepositoryEntryMode, name: &str, errors: &mut ValidationErrors) {
    if entry != RepositoryEntryMode::File {
        errors.push(format!(
            "file content assertion {name:?} must select file entries"
        ));
    }
}

fn unique(values: &[String], what: &str, errors: &mut ValidationErrors) {
    if values.iter().collect::<BTreeSet<_>>().len() != values.len() {
        errors.push(format!("{what} contains duplicate values"));
    }
}

pub(super) fn item_count(contract: &Contract) -> usize {
    contract
        .repository
        .files
        .iter()
        .map(|rule| {
            1 + rule.include.len()
                + rule.exclude.len()
                + match &rule.predicate {
                    RepositoryFilePredicate::ExactPaths { paths } => paths.len(),
                    RepositoryFilePredicate::ForbiddenNames { names, .. } => names.len(),
                    RepositoryFilePredicate::LiteralOrder { .. } => 2,
                    RepositoryFilePredicate::LiteralBetween { .. } => 3,
                    RepositoryFilePredicate::LineValuesAllowed { values, .. } => values.len() + 1,
                    RepositoryFilePredicate::Document(document) => {
                        document.path.len()
                            + match &document.assertion {
                                RepositoryDocumentAssertion::KeysExact { keys, .. }
                                | RepositoryDocumentAssertion::KeysAllowed { keys, .. } => {
                                    keys.len()
                                }
                                RepositoryDocumentAssertion::Equals {
                                    value: RepositoryDocumentValue::Strings(values),
                                } => values.len(),
                                _ => 1,
                            }
                    }
                    _ => 1,
                }
        })
        .sum()
}

#[cfg(test)]
#[path = "validate_files_test.rs"]
mod validate_files_test;
