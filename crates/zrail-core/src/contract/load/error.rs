//! Contract-load failures remain deterministic inspection text, not a wire protocol.

use std::{error::Error, fmt};

#[derive(Clone, Debug, Eq, PartialEq)]
/// One or more human-readable contract loading or validation failures.
/// Messages are deterministic for deterministic input, but are not a stable machine protocol.
pub struct ContractError {
    messages: Vec<String>,
}

impl ContractError {
    /// Creates an error containing exactly one human-readable message.
    pub fn one(message: impl Into<String>) -> Self {
        Self {
            messages: vec![message.into()],
        }
    }

    pub(crate) fn many(messages: Vec<String>) -> Self {
        Self { messages }
    }

    /// Returns all failure messages in their deterministic reporting order.
    pub fn messages(&self) -> &[String] {
        &self.messages
    }
}

impl fmt::Display for ContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.messages.join("\n"))
    }
}

impl Error for ContractError {}
