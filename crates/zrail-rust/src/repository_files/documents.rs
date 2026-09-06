//! Authored TOML/JSON predicates share file inputs, bounds, and exact policy diagnostics.

mod json;
mod keys;
mod node;

use zrail_core::{
    RepositoryDocumentAssertion as Assertion, RepositoryDocumentFormat, RepositoryDocumentPredicate,
};

use super::model::{GovernedRepositoryFileEntry, RepositoryDocumentObservation};
use node::Node;

pub(super) fn evaluate(
    policy: &RepositoryDocumentPredicate,
    bytes: &[u8],
    entry: &mut GovernedRepositoryFileEntry,
) -> Result<(), String> {
    let text =
        std::str::from_utf8(bytes).map_err(|error| format!("document is not UTF-8: {error}"))?;
    match policy.format {
        RepositoryDocumentFormat::Toml => {
            let value = text
                .parse::<toml::Value>()
                .map_err(|error| format!("invalid TOML: {error}"))?;
            inspect(policy, Node::Toml(&value), entry)
        }
        RepositoryDocumentFormat::Json => {
            let value = json::parse(text)?;
            inspect(policy, Node::Json(&value), entry)
        }
    }
}

fn inspect(
    policy: &RepositoryDocumentPredicate,
    root: Node<'_>,
    entry: &mut GovernedRepositoryFileEntry,
) -> Result<(), String> {
    root.check_bounds()?;
    let mut observation = RepositoryDocumentObservation {
        selected_type: None,
        value: None,
        value_omitted: false,
        selection_error: None,
        keys: None,
    };
    let selected = match root.select(&policy.path) {
        Ok(selected) => selected,
        Err(error) => {
            observation.selection_error = Some(error);
            entry.satisfied = false;
            entry.document = Some(observation);
            return Ok(());
        }
    };
    let value = selected.and_then(Node::value);
    entry.satisfied = match &policy.assertion {
        Assertion::Present => selected.is_some(),
        Assertion::Absent => selected.is_none(),
        Assertion::NonemptyString => selected
            .and_then(Node::string)
            .is_some_and(|value| !value.trim().is_empty()),
        Assertion::Equals { value: expected } => value.as_ref() == Some(expected),
        Assertion::KeysExact {
            keys: expected,
            empty_on_non_table,
        } => keys::evaluate(
            selected,
            expected,
            *empty_on_non_table,
            true,
            &mut observation,
        )?,
        Assertion::KeysAllowed {
            keys: expected,
            empty_on_non_table,
        } => keys::evaluate(
            selected,
            expected,
            *empty_on_non_table,
            false,
            &mut observation,
        )?,
    };
    observation.selected_type = selected.map(|node| node.kind().into());
    let encoded = value
        .as_ref()
        .map(serde_json::to_vec)
        .transpose()
        .map_err(|error| error.to_string())?;
    observation.value_omitted =
        selected.is_some() && encoded.as_ref().is_none_or(|bytes| bytes.len() > 16 * 1024);
    if !observation.value_omitted {
        observation.value = value;
    }
    entry.document = Some(observation);
    Ok(())
}
