//! Human-readable lock validation and persistence failures.

use std::{error::Error, fmt};

/// Human-readable lock parsing, validation, serialization, or I/O failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockError(pub(super) String);

impl fmt::Display for LockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for LockError {}

impl LockError {
    /// Creates an internal lock validation error without exposing its representation.
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}
