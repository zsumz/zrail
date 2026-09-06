//! Closed source-inventory claims distinguish authored syntax from compilation identity.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Required quantities over explicitly selected physical Rust files.
pub struct RustInventoryRule {
    /// Unique stable policy name.
    pub name: String,
    /// Human justification for preserving these quantities.
    pub reason: String,
    /// Repository-relative file patterns, independent of source discovery exclusions.
    pub include: Vec<String>,
    #[serde(default)]
    /// Explicit file patterns removed from this selection.
    pub exclude: Vec<String>,
    /// Mandatory interpretation of cfg branches and repeated source mounts.
    pub world: RustInventoryWorld,
    /// Closed source relationship and its written subject selection.
    pub subject: RustInventorySubject,
    /// Required quantity or complete per-file ownership set.
    pub assertion: RustInventoryAssertion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Compilation interpretation required by a Rust inventory.
pub enum RustInventoryWorld {
    /// Every authored cfg branch, once per physical syntax occurrence; no expansions.
    Authored,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
/// Source relationships whose quantity semantics are implemented by stock zrail.
pub enum RustInventorySubject {
    /// Written method identifiers, without resolving receiver types.
    WrittenMethods {
        /// Exact case-sensitive identifiers, including an authored `r#` prefix.
        names: Vec<String>,
    },
    /// Every authored expression path, including callees and function-value acquisition.
    WrittenExpressionPaths {
        /// Exact final two written identifiers; qualification and generic arguments are ignored.
        suffixes: Vec<String>,
    },
}

impl RustInventorySubject {
    /// Exact authored subject selectors, without semantic identity resolution.
    pub fn names(&self) -> &[String] {
        match self {
            Self::WrittenMethods { names } => names,
            Self::WrittenExpressionPaths { suffixes } => suffixes,
        }
    }

    /// Canonicalize unordered selectors without changing their authored spelling.
    pub fn canonicalize(&mut self) {
        match self {
            Self::WrittenMethods { names } => names.sort(),
            Self::WrittenExpressionPaths { suffixes } => suffixes.sort(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
/// Positive requirements and persistent zero-count prohibitions share exact observations.
pub enum RustInventoryAssertion {
    /// Inclusive bounds over all selected logical syntax occurrences.
    Count {
        #[serde(default)]
        /// Required minimum, including zero for a prohibition.
        minimum: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Optional inclusive maximum.
        maximum: Option<usize>,
    },
    /// Complete per-file/per-subject map; every unlisted pair must have zero occurrences.
    ExactCounts {
        /// Unique positive counts; an empty map prohibits all selected occurrences.
        counts: Vec<RustInventoryCount>,
    },
    /// Complete file/subject set; duplicates at an existing owner do not change membership.
    ExactOwners {
        /// Every listed pair must occur; unlisted pairs are forbidden. Empty prohibits all.
        owners: Vec<RustInventoryOwner>,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// A required ownership location, independent of occurrence quantity at that location.
pub struct RustInventoryOwner {
    /// Exact normalized repository-relative Rust file.
    pub path: String,
    /// Exact written subject selector.
    pub name: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// A physical ownership location and its distinct occurrence quantity.
pub struct RustInventoryCount {
    /// Exact normalized repository-relative Rust file.
    pub path: String,
    /// Exact written subject selector, including both segments for expression paths.
    pub name: String,
    /// Positive number of distinct physical syntax occurrences.
    pub count: usize,
}
