//! Validation and suggestions for concrete explanation paths.

use std::path::Path;

use zrail_core::normalize_relative;

use crate::engine::{CheckError, RepositoryModel};

pub(super) fn existing(model: &RepositoryModel, path: &Path) -> Result<String, CheckError> {
    if path.is_absolute() {
        if !path_exists(path)? {
            return Err(missing(model, path, "an absolute path"));
        }
        return Err(CheckError::from_message(format!(
            "absolute path is not repository-relative: {}",
            path.display()
        )));
    }

    let relative = normalize_relative(path).map_err(CheckError::from_message)?;
    let absolute = model.inventory.root.join(&relative);
    if !path_exists(&absolute)? {
        return Err(missing(model, path, "repository-relative"));
    }
    Ok(relative)
}

pub(super) fn hypothetical(path: &Path) -> Result<String, CheckError> {
    normalize_relative(path).map_err(CheckError::from_message)
}

fn path_exists(path: &Path) -> Result<bool, CheckError> {
    path.try_exists().map_err(|error| {
        CheckError::from_message(format!(
            "check whether path exists at {}: {error}",
            path.display()
        ))
    })
}

fn missing(model: &RepositoryModel, path: &Path, interpretation: &str) -> CheckError {
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        model.inventory.root.join(path)
    };
    let mut message = format!(
        "path does not exist: {}\ninterpreted as {interpretation}: {}",
        path.display(),
        resolved.display()
    );
    let suggestions = close_matches(model, path);
    if !suggestions.is_empty() {
        message.push_str("\nclose matches:");
        for suggestion in suggestions {
            message.push_str("\n  ");
            message.push_str(suggestion);
        }
    }
    message.push_str(
        "\nclassify a path that does not exist with \
         `zrail explain --hypothetical-path <repository-relative-path>`",
    );
    CheckError::from_message(message)
}

fn close_matches<'a>(model: &'a RepositoryModel, path: &Path) -> Vec<&'a str> {
    let target = suggestion_target(model, path);
    let target_name = Path::new(&target)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(&target);
    let threshold = (target.chars().count() / 3).clamp(2, 12);
    let mut candidates = model
        .inventory
        .entries
        .iter()
        .filter_map(|entry| {
            let candidate_name = Path::new(&entry.relative)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(&entry.relative);
            let full = edit_distance(&target, &entry.relative);
            let name = edit_distance(target_name, candidate_name).saturating_add(2);
            let score = full.min(name);
            (score <= threshold).then_some((score, entry.relative.as_str()))
        })
        .collect::<Vec<_>>();
    candidates.sort_unstable();
    candidates.dedup_by_key(|candidate| candidate.1);
    candidates
        .into_iter()
        .take(3)
        .map(|(_, path)| path)
        .collect()
}

fn suggestion_target(model: &RepositoryModel, path: &Path) -> String {
    if path.is_absolute() {
        path.strip_prefix(&model.inventory.root)
            .ok()
            .and_then(|relative| relative.to_str())
            .unwrap_or_else(|| path.to_str().unwrap_or_default())
            .replace('\\', "/")
    } else {
        path.to_str().unwrap_or_default().replace('\\', "/")
    }
}

fn edit_distance(left: &str, right: &str) -> usize {
    let right = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    let mut current = vec![0; right.len() + 1];
    for (left_index, left_character) in left.chars().enumerate() {
        current[0] = left_index + 1;
        for (right_index, right_character) in right.iter().enumerate() {
            current[right_index + 1] = (previous[right_index + 1] + 1)
                .min(current[right_index] + 1)
                .min(previous[right_index] + usize::from(left_character != *right_character));
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[right.len()]
}

#[cfg(test)]
#[path = "path_input_test.rs"]
mod path_input_test;
