//! Closed content predicates bound every authored marker, key, and expected value.

use crate::{
    RepositoryDocumentAssertion, RepositoryDocumentPredicate, RepositoryDocumentValue,
    RepositoryFilePredicate, RepositoryTextNormalization,
};

use super::{ValidationErrors, unique};

pub(super) fn validate(predicate: &RepositoryFilePredicate, errors: &mut ValidationErrors) {
    match predicate {
        RepositoryFilePredicate::Document(document) => validate_document(document, errors),
        RepositoryFilePredicate::LiteralOrder { before, after } => {
            markers(&[before, after], errors);
        }
        RepositoryFilePredicate::LiteralBetween {
            start,
            end,
            contains,
        } => {
            markers(&[start, end, contains], errors);
        }
        RepositoryFilePredicate::LineValuesAllowed {
            prefix,
            values,
            normalization,
        } => {
            markers(&[prefix], errors);
            unique(values, "line value sets", errors);
            if values.len() > 1_024 || values.iter().map(String::len).sum::<usize>() > 16 * 1_024 {
                errors.push(
                    "line value sets permit at most 1024 values and 16384 value bytes".into(),
                );
            }
            if std::iter::once(prefix)
                .chain(values)
                .any(|value| value.contains(['\n', '\r']))
            {
                errors.push("line value prefixes and values cannot contain CR or LF".into());
            }
            if *normalization == RepositoryTextNormalization::RemoveWhitespace {
                errors.push(
                    "line value selection supports none, trim-start, or trim normalization".into(),
                );
            }
        }
        RepositoryFilePredicate::LinePrefixesAbsent {
            prefixes,
            normalization,
        } => {
            unique(prefixes, "line prefix sets", errors);
            if prefixes.is_empty()
                || prefixes.len() > 64
                || prefixes.iter().map(String::len).sum::<usize>() > 16 * 1_024
            {
                errors
                    .push("line prefix sets require 1..64 prefixes and at most 16384 bytes".into());
            }
            if prefixes.iter().any(|prefix| {
                prefix.is_empty()
                    || prefix.contains(['\n', '\r'])
                    || (*normalization != RepositoryTextNormalization::None
                        && prefix.trim_start() != prefix)
            }) {
                errors.push("line prefixes must be nonempty, contain no CR/LF, and satisfy leading normalization".into());
            }
            if *normalization == RepositoryTextNormalization::RemoveWhitespace {
                errors.push("line prefixes support none, trim-start, or trim normalization".into());
            }
        }
        _ => {}
    }
}

fn markers(values: &[&String], errors: &mut ValidationErrors) {
    if values
        .iter()
        .any(|value| value.is_empty() || value.len() > 16 * 1_024)
    {
        errors.push("raw structure markers require 1..16384 UTF-8 bytes each".into());
    }
}

fn validate_document(document: &RepositoryDocumentPredicate, errors: &mut ValidationErrors) {
    if let RepositoryDocumentAssertion::FieldNotString { field } = &document.assertion
        && field.len() > 1_024
    {
        errors.push("document field projections permit at most 1024 key bytes".into());
    }
    if document.path.len() > 32 || document.path.iter().any(|key| key.len() > 1_024) {
        errors.push(
            "document paths permit at most 32 literal keys of at most 1024 bytes each".into(),
        );
    }
    if let RepositoryDocumentAssertion::KeysExact { keys, .. }
    | RepositoryDocumentAssertion::KeysAllowed { keys, .. } = &document.assertion
    {
        unique(keys, "document key sets", errors);
        if keys.len() > 1_024 || keys.iter().map(String::len).sum::<usize>() > 16 * 1_024 {
            errors.push("document key sets permit at most 1024 keys and 16384 key bytes".into());
        }
    }
    if let RepositoryDocumentAssertion::Equals { value } = &document.assertion {
        let (count, bytes) = match value {
            RepositoryDocumentValue::String(value) => (1, value.len()),
            RepositoryDocumentValue::Strings(values) => {
                (values.len(), values.iter().map(String::len).sum())
            }
            _ => (1, 0),
        };
        if count > 1_024 || bytes > 16 * 1_024 {
            errors.push(
                "document equality permits at most 1024 strings and 16384 expected string bytes"
                    .into(),
            );
        }
    }
}

#[cfg(test)]
#[path = "content_test.rs"]
mod content_test;
