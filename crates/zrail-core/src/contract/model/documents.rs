//! Closed authored-document checks use literal key paths and typed expected values.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One structural document predicate, independent of Cargo resolution and execution.
pub struct RepositoryDocumentPredicate {
    /// Explicit parser; format is never inferred from a file extension.
    pub format: RepositoryDocumentFormat,
    /// Literal case-sensitive object/table keys, from the document root.
    /// An empty path selects the root. Dots and other punctuation are literal.
    pub path: Vec<String>,
    /// Closed condition on the selected value.
    pub assertion: RepositoryDocumentAssertion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Supported deterministic document parsers; no aliases or expression evaluation.
pub enum RepositoryDocumentFormat {
    /// TOML 1.0, retaining authored types and rejecting duplicate keys.
    Toml,
    /// JSON, rejecting duplicate object keys and nonstandard syntax.
    Json,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "kebab-case", deny_unknown_fields)]
/// Conditions never coerce types or search unrelated document locations.
pub enum RepositoryDocumentAssertion {
    /// The selected key exists, including a JSON null value.
    Present,
    /// The selected key is absent; zero selected files remains a valid prohibition.
    Absent,
    /// The selected value is a string nonempty after Rust's Unicode-aware trim.
    NonemptyString,
    /// Exact typed value; string arrays preserve order, duplicates, and cardinality.
    Equals {
        /// Expected scalar or complete ordered string array.
        value: RepositoryDocumentValue,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
/// The initially inventoried exact document values; unsupported expectations fail parsing.
pub enum RepositoryDocumentValue {
    /// Case-sensitive UTF-8 string, without trimming or normalization.
    String(String),
    /// Boolean, distinct from its string spelling.
    Boolean(bool),
    /// Signed 64-bit integer, distinct from floating-point values.
    Integer(i64),
    /// Complete ordered string array, including an empty array.
    Strings(Vec<String>),
}
