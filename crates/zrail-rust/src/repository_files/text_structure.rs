//! First-marker intervals and prefixed line sets reuse bounded immutable file bytes.

use std::collections::BTreeSet;

use zrail_core::{RepositoryFilePredicate, RepositoryTextNormalization};

use super::{
    GovernedRepositoryFileEntry, RepositoryLineValue,
    RepositoryTextStructureObservation as Observation,
};

pub(super) fn evaluate(
    predicate: &RepositoryFilePredicate,
    bytes: &[u8],
    entry: &mut GovernedRepositoryFileEntry,
) -> Result<(), String> {
    let source = std::str::from_utf8(bytes)
        .map_err(|error| format!("raw structure input {:?} is not UTF-8: {error}", entry.path))?;
    entry.valid_utf8 = Some(true);
    let observation = match predicate {
        RepositoryFilePredicate::LiteralOrder { before, after } => {
            let before = source.find(before);
            let after = source.find(after);
            entry.satisfied = before
                .zip(after)
                .is_some_and(|(before, after)| before < after);
            Observation::Order { before, after }
        }
        RepositoryFilePredicate::LiteralBetween {
            start,
            end,
            contains,
        } => {
            let start = source.find(start);
            let end = source.find(end);
            let interval = start.zip(end).filter(|(start, end)| start < end);
            let mut count = 0;
            if let Some((start, end)) = interval {
                for (offset, _) in source[start..end].match_indices(contains) {
                    count += 1;
                    if entry.literal_offsets.len() < 16 {
                        entry.literal_offsets.push(start + offset);
                    }
                }
            }
            entry.literal_count = Some(count);
            entry.omitted_literal_offsets = count - entry.literal_offsets.len();
            entry.satisfied = interval.is_some() && count > 0;
            Observation::Between { start, end }
        }
        RepositoryFilePredicate::LineValuesAllowed {
            prefix,
            values,
            normalization,
        } => {
            let observation = line_values(source, prefix, values, *normalization)?;
            entry.satisfied = matches!(
                observation,
                Observation::LineValues {
                    unauthorized_count: 0,
                    ..
                }
            );
            observation
        }
        RepositoryFilePredicate::LinePrefixesAbsent {
            prefixes,
            normalization,
        } => {
            let mut forbidden_count = 0;
            let mut forbidden_lines = Vec::new();
            for (index, line) in source.lines().enumerate() {
                let line = normalize_line(line, *normalization)?;
                if prefixes.iter().any(|prefix| line.starts_with(prefix)) {
                    forbidden_count += 1;
                    if forbidden_lines.len() < 16 {
                        forbidden_lines.push(index + 1);
                    }
                }
            }
            entry.satisfied = forbidden_count == 0;
            Observation::LinePrefixes {
                forbidden_count,
                forbidden_omitted: forbidden_count - forbidden_lines.len(),
                forbidden_lines,
            }
        }
        _ => return Err("unsupported raw structure predicate".into()),
    };
    entry.text_structure = Some(observation);
    Ok(())
}

fn line_values(
    source: &str,
    prefix: &str,
    values: &[String],
    normalization: RepositoryTextNormalization,
) -> Result<Observation, String> {
    let allowed = values.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let mut selected_count = 0;
    let mut unauthorized_count = 0;
    let mut unauthorized_sample = Vec::new();
    let mut sample_bytes = 2;
    for (index, line) in source.lines().enumerate() {
        let line = normalize_line(line, normalization)?;
        let Some(value) = line.strip_prefix(prefix) else {
            continue;
        };
        selected_count += 1;
        if allowed.contains(value) {
            continue;
        }
        unauthorized_count += 1;
        if unauthorized_sample.len() == 16 || value.len() > 16 * 1024 {
            continue;
        }
        let sample = RepositoryLineValue {
            line: index + 1,
            value: value.into(),
        };
        let bytes = serde_json::to_vec(&sample)
            .map_err(|error| error.to_string())?
            .len()
            + usize::from(!unauthorized_sample.is_empty());
        if sample_bytes + bytes <= 16 * 1024 {
            sample_bytes += bytes;
            unauthorized_sample.push(sample);
        }
    }
    Ok(Observation::LineValues {
        selected_count,
        unauthorized_count,
        unauthorized_omitted: unauthorized_count - unauthorized_sample.len(),
        unauthorized_sample,
    })
}

fn normalize_line(line: &str, normalization: RepositoryTextNormalization) -> Result<&str, String> {
    match normalization {
        RepositoryTextNormalization::None => Ok(line),
        RepositoryTextNormalization::TrimStart => Ok(line.trim_start()),
        RepositoryTextNormalization::Trim => Ok(line.trim()),
        RepositoryTextNormalization::RemoveWhitespace => {
            Err("unsupported raw line normalization".into())
        }
    }
}

#[cfg(test)]
#[path = "text_structure_test.rs"]
mod text_structure_test;
