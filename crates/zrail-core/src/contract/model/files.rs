//! Closed repository path and raw-file predicates, separate from Rust and execution claims.

use super::RepositoryDocumentPredicate;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One named assertion over a bounded selection of repository entries.
pub struct RepositoryFileRule {
    /// Stable policy identity, independent of TOML order.
    pub name: String,
    /// Repository-relative patterns selecting entries; literals also inspect absent paths.
    pub include: Vec<String>,
    /// Explicit per-rule subtraction, independent of Rust source exclusions.
    #[serde(default)]
    pub exclude: Vec<String>,
    /// Entry kind used for selection.
    #[serde(default)]
    pub entry: RepositoryEntryMode,
    /// Closed predicate evaluated over the selected entries.
    pub predicate: RepositoryFilePredicate,
    /// Human justification for the repository contract.
    pub reason: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Entry kinds selected by a repository assertion.
pub enum RepositoryEntryMode {
    /// Regular files, including explicitly resolved contained file links.
    #[default]
    File,
    /// Directories, without recursively following directory links.
    Directory,
    /// Non-directory entries, including special files and resolved contained links.
    NonDirectory,
    /// Any filesystem entry at a selected path, including a broken link.
    Any,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
/// Bounded file predicates do not interpret code or execute repository programs.
pub enum RepositoryFilePredicate {
    /// Count distinct selected physical paths; zero is a persistent prohibition.
    Count {
        /// Minimum selected paths, including zero for a prohibition.
        minimum: usize,
        /// Optional inclusive upper bound.
        #[serde(default)]
        maximum: Option<usize>,
    },
    /// Require the complete selected path set, including an explicitly empty set.
    ExactPaths {
        /// Exact normalized paths; ordering has no semantic effect.
        paths: Vec<String>,
    },
    /// Reject literal names in selected paths; no regex or semantic name resolution.
    ForbiddenNames {
        /// Literal names whose presence is forbidden.
        names: Vec<String>,
        /// Which written path elements are compared.
        part: RepositoryNamePart,
        /// Relative source names or explicitly requested filesystem path spelling.
        #[serde(default)]
        basis: RepositoryNameBasis,
        /// Literal comparison case semantics.
        #[serde(default)]
        case: RepositoryCaseMode,
    },
    /// Require each selected file to satisfy an explicitly raw UTF-8 text predicate.
    Literal(RepositoryLiteralPredicate),
    /// Require both case-sensitive markers and strict ordering of their first byte offsets.
    LiteralOrder {
        /// First occurrence must precede the first occurrence of `after`.
        before: String,
        /// Required later marker; comments and strings participate deliberately.
        after: String,
    },
    /// Require a literal inside the interval between the first two case-sensitive markers.
    LiteralBetween {
        /// Required first occurrence; its byte position is included in the interval.
        start: String,
        /// Required first occurrence; its byte position is excluded from the interval.
        end: String,
        /// Required literal in the selected interval; never code or a regular expression.
        contains: String,
    },
    /// Check each raw line whose normalized spelling begins with one literal prefix.
    LineValuesAllowed {
        /// Nonempty case-sensitive prefix selecting a line and removed before comparison.
        prefix: String,
        /// Complete allowed suffix values; an empty set prohibits every selected line.
        values: Vec<String>,
        /// Transform each line before prefix selection; comparison remains case-sensitive.
        #[serde(default)]
        normalization: RepositoryTextNormalization,
    },
    /// Forbid lines beginning with any member of a bounded literal prefix set.
    LinePrefixesAbsent {
        /// Nonempty, distinct, case-sensitive prefixes; overlap never duplicates a line.
        prefixes: Vec<String>,
        /// Transform each physical line before prefix matching.
        #[serde(default)]
        normalization: RepositoryTextNormalization,
    },
    /// Parse a bounded authored document and inspect one literal key path.
    Document(RepositoryDocumentPredicate),
    /// Require at least one selected file and exact bytes equal to the reference.
    BytesEqual {
        /// Exact repository-relative regular-file reference, also bound as an input.
        other: String,
        /// Additionally require valid UTF-8 on both sides; binary equality is the default.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        utf8: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Literal text matching deliberately includes comments and string contents.
pub struct RepositoryLiteralPredicate {
    /// Literal text; never a regex, command, or Rust expression.
    pub text: String,
    /// Relation checked independently in every selected file.
    pub mode: RepositoryLiteralMode,
    /// Required only for exact-count mode; Rust non-overlapping string occurrences.
    #[serde(default)]
    pub count: Option<usize>,
    /// Explicit transformation of input text before matching.
    #[serde(default)]
    pub normalization: RepositoryTextNormalization,
    /// Case semantics for the literal and transformed input.
    #[serde(default)]
    pub case: RepositoryCaseMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Positive literal requirements also require a nonempty file selection.
pub enum RepositoryLiteralMode {
    /// At least one non-overlapping occurrence per selected file.
    Contains,
    /// Zero occurrences; a missing prohibited occurrence is valid policy.
    Absent,
    /// The transformed file starts with the literal.
    StartsWith,
    /// The transformed file ends with the literal.
    EndsWith,
    /// The transformed file equals the literal.
    Equals,
    /// Exactly `count` non-overlapping occurrences per selected file.
    ExactCount,
    /// At least one complete line equals the literal after per-line normalization.
    LinePresent,
    /// No complete line equals the literal; absence remains a live prohibition.
    LineAbsent,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Closed raw-text transformations; none interprets code or document structure.
pub enum RepositoryTextNormalization {
    /// Preserve the input text byte-for-byte after UTF-8 decoding.
    #[default]
    None,
    /// Apply Rust's Unicode-aware `str::trim_start` to the input.
    TrimStart,
    /// Apply Rust's Unicode-aware `str::trim` to the input or each selected line.
    Trim,
    /// Remove characters matching Rust's Unicode-aware `char::is_whitespace`.
    RemoveWhitespace,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Explicit literal case comparison without locale-dependent behavior.
pub enum RepositoryCaseMode {
    /// Compare exactly, including case and UTF-8 spelling.
    #[default]
    Sensitive,
    /// Fold ASCII letters only; non-ASCII text remains unchanged.
    AsciiInsensitive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Closed choices for path-name restrictions.
pub enum RepositoryNamePart {
    /// Every full path-component name.
    Component,
    /// Rust `Path::file_stem` for every component, including directory components.
    ComponentStem,
    /// The final file name including its extension.
    FileName,
    /// The final file name without its last extension.
    FileStem,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Name checks normally govern portable repository-relative paths.
pub enum RepositoryNameBasis {
    /// Inspect only the selected repository-relative path.
    #[default]
    Repository,
    /// Include the actual checkout path; that context must be exposed and bound.
    Filesystem,
}
