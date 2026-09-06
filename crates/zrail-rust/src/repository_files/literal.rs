//! Deliberately raw text matching with explicit Unicode whitespace and ASCII case rules.

use std::borrow::Cow;

use zrail_core::{
    RepositoryCaseMode, RepositoryLiteralMode, RepositoryLiteralPredicate,
    RepositoryTextNormalization,
};

use super::model::GovernedRepositoryFileEntry;

pub(super) fn requires_file(policy: &RepositoryLiteralPredicate) -> bool {
    policy.mode != RepositoryLiteralMode::Absent
        && !(policy.mode == RepositoryLiteralMode::ExactCount && policy.count == Some(0))
}

pub(super) fn evaluate(
    policy: &RepositoryLiteralPredicate,
    bytes: &[u8],
    observation: &mut GovernedRepositoryFileEntry,
) -> Result<(), String> {
    let source = std::str::from_utf8(bytes).map_err(|error| {
        format!(
            "raw text input {:?} is not UTF-8: {error}",
            observation.path
        )
    })?;
    let source = match policy.normalization {
        RepositoryTextNormalization::None => Cow::Borrowed(source),
        RepositoryTextNormalization::TrimStart => Cow::Borrowed(source.trim_start()),
        RepositoryTextNormalization::RemoveWhitespace => Cow::Owned(
            source
                .chars()
                .filter(|character| !character.is_whitespace())
                .collect(),
        ),
    };
    let source = fold(&source, policy.case);
    let literal = fold(&policy.text, policy.case);
    let mut count = 0;
    for (offset, _) in source.match_indices(literal.as_ref()) {
        count += 1;
        if observation.literal_offsets.len() < 16 {
            observation.literal_offsets.push(offset);
        }
    }
    observation.literal_count = Some(count);
    observation.omitted_literal_offsets = count - observation.literal_offsets.len();
    observation.satisfied = match policy.mode {
        RepositoryLiteralMode::Contains => count > 0,
        RepositoryLiteralMode::Absent => count == 0,
        RepositoryLiteralMode::StartsWith => source.starts_with(literal.as_ref()),
        RepositoryLiteralMode::EndsWith => source.ends_with(literal.as_ref()),
        RepositoryLiteralMode::Equals => source == literal,
        RepositoryLiteralMode::ExactCount => policy.count == Some(count),
    };
    Ok(())
}

pub(super) fn fold(value: &str, case: RepositoryCaseMode) -> Cow<'_, str> {
    match case {
        RepositoryCaseMode::Sensitive => Cow::Borrowed(value),
        RepositoryCaseMode::AsciiInsensitive => Cow::Owned(value.to_ascii_lowercase()),
    }
}
