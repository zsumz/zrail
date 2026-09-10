//! Deliberately raw text matching with explicit Unicode whitespace and ASCII case rules.

use std::borrow::Cow;

use zrail_core::{
    RepositoryCaseMode, RepositoryLiteralMode, RepositoryLiteralPredicate,
    RepositoryTextNormalization,
};

use super::model::GovernedRepositoryFileEntry;

pub(super) fn requires_file(policy: &RepositoryLiteralPredicate) -> bool {
    !(matches!(
        policy.mode,
        RepositoryLiteralMode::Absent | RepositoryLiteralMode::LineAbsent
    ) || policy.mode == RepositoryLiteralMode::ExactCount && policy.count == Some(0))
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
    let lines = matches!(
        policy.mode,
        RepositoryLiteralMode::LinePresent | RepositoryLiteralMode::LineAbsent
    );
    let source = if lines {
        let mut transformed = String::with_capacity(source.len());
        for (index, line) in source.lines().enumerate() {
            if index > 0 {
                transformed.push('\n');
            }
            transformed.push_str(&normalize(line, policy.normalization));
        }
        Cow::Owned(transformed)
    } else {
        normalize(source, policy.normalization)
    };
    let source = fold(&source, policy.case);
    let literal = fold(&policy.text, policy.case);
    let mut count = 0;
    let mut record = |offset| {
        count += 1;
        if observation.literal_offsets.len() < 16 {
            observation.literal_offsets.push(offset);
        }
    };
    if lines {
        let mut offset = 0;
        for line in source.split('\n') {
            if line == literal {
                record(offset);
            }
            offset += line.len() + 1;
        }
    } else {
        for (offset, _) in source.match_indices(literal.as_ref()) {
            record(offset);
        }
    }
    observation.literal_count = Some(count);
    observation.omitted_literal_offsets = count - observation.literal_offsets.len();
    observation.satisfied = match policy.mode {
        RepositoryLiteralMode::Contains | RepositoryLiteralMode::LinePresent => count > 0,
        RepositoryLiteralMode::Absent | RepositoryLiteralMode::LineAbsent => count == 0,
        RepositoryLiteralMode::StartsWith => source.starts_with(literal.as_ref()),
        RepositoryLiteralMode::EndsWith => source.ends_with(literal.as_ref()),
        RepositoryLiteralMode::Equals => source == literal,
        RepositoryLiteralMode::ExactCount => policy.count == Some(count),
    };
    Ok(())
}

fn normalize(source: &str, mode: RepositoryTextNormalization) -> Cow<'_, str> {
    match mode {
        RepositoryTextNormalization::None => Cow::Borrowed(source),
        RepositoryTextNormalization::TrimStart => Cow::Borrowed(source.trim_start()),
        RepositoryTextNormalization::Trim => Cow::Borrowed(source.trim()),
        RepositoryTextNormalization::RemoveWhitespace => Cow::Owned(
            source
                .chars()
                .filter(|character| !character.is_whitespace())
                .collect(),
        ),
    }
}

pub(super) fn fold(value: &str, case: RepositoryCaseMode) -> Cow<'_, str> {
    match case {
        RepositoryCaseMode::Sensitive => Cow::Borrowed(value),
        RepositoryCaseMode::AsciiInsensitive => Cow::Owned(value.to_ascii_lowercase()),
    }
}
