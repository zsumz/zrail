//! Initialization selection is normalized before any repository traversal.

use std::path::Path;

use zrail_core::{glob_matches, normalize_relative};

use crate::engine::CheckError;

const MAX_PATTERN_BYTES: usize = 4 * 1024;
const MAX_PATTERN_SEGMENTS: usize = 256;

/// Canonical repository exclusions used during onboarding discovery.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RepositorySelection {
    exclusions: Vec<String>,
}

impl RepositorySelection {
    /// Normalizes, sorts, and deduplicates repository-relative exclusion patterns.
    pub fn new(exclusions: impl IntoIterator<Item = String>) -> Result<Self, CheckError> {
        let mut exclusions = exclusions
            .into_iter()
            .map(|pattern| normalize(&pattern))
            .collect::<Result<Vec<_>, _>>()?;
        exclusions.sort();
        exclusions.dedup();
        Ok(Self { exclusions })
    }

    /// Returns canonical exclusions in deterministic order.
    pub fn exclusions(&self) -> &[String] {
        &self.exclusions
    }

    pub(crate) fn matching_exclusion(&self, relative: &str) -> Option<&str> {
        self.exclusions.iter().find_map(|pattern| {
            (glob_matches(pattern, relative) || relative.starts_with(&format!("{pattern}/")))
                .then_some(pattern.as_str())
        })
    }
}

fn normalize(pattern: &str) -> Result<String, CheckError> {
    let pattern = pattern.trim();
    if pattern.starts_with('!') {
        return Err(CheckError::from_message(format!(
            "repository exclusion {pattern:?} uses unsupported negation"
        )));
    }
    let normalized = normalize_relative(Path::new(pattern)).map_err(CheckError::from_message)?;
    if normalized.is_empty() {
        return Err(CheckError::from_message(
            "repository exclusion may not be empty",
        ));
    }
    if normalized.len() > MAX_PATTERN_BYTES || normalized.split('/').count() > MAX_PATTERN_SEGMENTS
    {
        return Err(CheckError::from_message(format!(
            "repository exclusion exceeds the {MAX_PATTERN_BYTES}-byte or {MAX_PATTERN_SEGMENTS}-segment safety limit: {normalized:?}"
        )));
    }
    Ok(normalized)
}

#[cfg(test)]
#[path = "selection_test.rs"]
mod selection_test;
