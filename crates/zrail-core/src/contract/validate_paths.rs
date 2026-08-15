//! Canonical repository paths and exact Cargo package identifiers.

use std::path::{Component, Path};

use crate::path::{MAX_GLOB_PATTERN_BYTES, MAX_GLOB_PATTERN_SEGMENTS};

use super::validate_limits::ValidationErrors;

pub(super) fn validate_repository_literal(value: &str, errors: &mut ValidationErrors) {
    if value == "." {
        return;
    }
    validate_repository_pattern(value, errors);
    if has_wildcard(value) {
        errors.push(format!(
            "expected an exact repository path, found pattern {value:?}"
        ));
    }
}

pub(super) fn validate_repository_pattern(value: &str, errors: &mut ValidationErrors) {
    if value.len() > MAX_GLOB_PATTERN_BYTES || value.split('/').count() > MAX_GLOB_PATTERN_SEGMENTS
    {
        errors.push(format!(
            "repository path pattern exceeds the {MAX_GLOB_PATTERN_BYTES}-byte or {MAX_GLOB_PATTERN_SEGMENTS}-segment safety limit: {value:?}"
        ));
        return;
    }
    if value.trim().is_empty() || value.contains('\\') || Path::new(value).is_absolute() {
        errors.push(format!(
            "invalid repository-relative path or pattern {value:?}"
        ));
        return;
    }
    for component in Path::new(value).components() {
        if matches!(
            component,
            Component::CurDir | Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) {
            errors.push(format!("path or pattern is not canonical: {value:?}"));
            return;
        }
    }
}

pub(super) fn validate_package_pattern(value: &str, errors: &mut ValidationErrors) {
    if value.len() > MAX_GLOB_PATTERN_BYTES
        || value.trim().is_empty()
        || value.chars().any(char::is_whitespace)
        || value
            .bytes()
            .any(|byte| !byte.is_ascii_alphanumeric() && !matches!(byte, b'-' | b'_' | b'*' | b'?'))
    {
        errors.push(format!("invalid package selector {value:?}"));
    }
}

pub(super) fn validate_package_name(value: &str, errors: &mut ValidationErrors) {
    if value.trim().is_empty()
        || value.chars().any(char::is_whitespace)
        || value
            .bytes()
            .any(|byte| matches!(byte, b'*' | b'?' | b'/' | b'\\'))
    {
        errors.push(format!("invalid exact package name {value:?}"));
    }
}

fn has_wildcard(value: &str) -> bool {
    value.bytes().any(|byte| matches!(byte, b'*' | b'?'))
}

#[cfg(test)]
#[path = "validate_paths_test.rs"]
mod validate_paths_test;
