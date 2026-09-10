//! Immediate authored key sets preserve exactness and explicit legacy type projections.

use std::collections::BTreeSet;

use super::{RepositoryDocumentObservation, node::Node};
use crate::repository_files::model::RepositoryDocumentKeys;

pub(super) fn evaluate(
    selected: Option<Node<'_>>,
    expected: &[String],
    empty_on_non_table: bool,
    exact: bool,
    observation: &mut RepositoryDocumentObservation,
) -> Result<bool, String> {
    let Some(selected) = selected else {
        return Ok(false);
    };
    let actual = match selected.keys() {
        Some(keys) => keys,
        None if empty_on_non_table => Vec::new(),
        None => {
            observation.selection_error = Some(format!(
                "key set requires an object/table, found {}",
                selected.kind()
            ));
            return Ok(false);
        }
    }
    .into_iter()
    .collect::<BTreeSet<_>>();
    let expected = expected.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let unexpected = actual.difference(&expected).copied().collect::<Vec<_>>();
    let missing = if exact {
        expected
            .difference(&actual)
            .map(|key| (*key).to_owned())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let satisfied = unexpected.is_empty() && missing.is_empty();
    let key_sample = sample(actual.iter().copied())?;
    let unexpected_sample = sample(unexpected.iter().copied())?;
    observation.keys = Some(RepositoryDocumentKeys {
        count: actual.len(),
        omitted: actual.len() - key_sample.len(),
        sample: key_sample,
        missing,
        unexpected_count: unexpected.len(),
        unexpected_omitted: unexpected.len() - unexpected_sample.len(),
        unexpected_sample,
    });
    Ok(satisfied)
}

fn sample<'a>(keys: impl Iterator<Item = &'a str>) -> Result<Vec<String>, String> {
    let mut bytes = 0;
    let mut sample = Vec::new();
    for key in keys {
        let size = serde_json::to_vec(key)
            .map_err(|error| error.to_string())?
            .len();
        if sample.len() < 16 && bytes + size <= 16 * 1024 {
            sample.push(key.into());
            bytes += size;
        }
    }
    Ok(sample)
}
