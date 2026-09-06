//! Raw structure observations expose exact positions and totals without execution claims.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
/// Complete matching boundaries for a closed raw-text predicate.
pub enum RepositoryTextStructureObservation {
    /// Original UTF-8 byte offsets of the first required marker occurrences.
    Order {
        /// First occurrence of the required earlier marker; absent means failure.
        before: Option<usize>,
        /// First occurrence of the required later marker; absent means failure.
        after: Option<usize>,
    },
    /// Original UTF-8 byte interval; literal counts and offsets are on the entry.
    Between {
        /// Inclusive first start-marker byte offset; absent means failure.
        start: Option<usize>,
        /// Exclusive first end-marker byte offset; absent means failure.
        end: Option<usize>,
    },
    /// Full quantities after per-line normalization and literal prefix selection.
    LineValues {
        /// Every matching line, including repeated equal values.
        selected_count: usize,
        /// Every selected line whose complete suffix is outside the allowed set.
        unauthorized_count: usize,
        /// At most sixteen unauthorized lines and 16 KiB of JSON-encoded sample bytes.
        unauthorized_sample: Vec<RepositoryLineValue>,
        /// Unauthorized lines omitted only from display, never from evaluation.
        unauthorized_omitted: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One complete unauthorized suffix, with its original one-based physical line.
pub struct RepositoryLineValue {
    /// Physical line number before normalization.
    pub line: usize,
    /// Complete normalized suffix after removing the literal selection prefix.
    pub value: String,
}
