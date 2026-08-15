//! Repository-relative path normalization and bounded glob matching.

use std::{
    fs,
    path::{Component, Path, PathBuf},
};

pub fn normalize_relative(path: &Path) -> Result<String, String> {
    if path.is_absolute() {
        return Err(format!(
            "absolute path is not repository-relative: {}",
            path.display()
        ));
    }
    let rendered = path
        .to_str()
        .ok_or_else(|| format!("path is not valid UTF-8: {}", path.display()))?;
    if rendered.contains('\\') {
        return Err(format!(
            "path uses a platform-dependent separator: {}",
            path.display()
        ));
    }
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(value) => {
                let value = value.to_str().ok_or_else(|| {
                    format!("path component is not valid UTF-8: {}", path.display())
                })?;
                parts.push(value.to_owned());
            }
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(format!("path escapes repository: {}", path.display()));
            }
        }
    }
    Ok(parts.join("/"))
}

pub fn repository_relative(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path.strip_prefix(root).map_err(|_| {
        format!(
            "path is outside repository {}: {}",
            root.display(),
            path.display()
        )
    })?;
    let mut parts = Vec::new();
    for component in relative.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(value) => {
                let value = value.to_str().ok_or_else(|| {
                    format!(
                        "repository path component is not valid UTF-8: {}",
                        path.display()
                    )
                })?;
                if value.contains(['/', '\\']) {
                    return Err(format!(
                        "repository path component is not portable: {}",
                        path.display()
                    ));
                }
                parts.push(value.to_owned());
            }
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(format!(
                    "repository path is not relative to its root: {}",
                    path.display()
                ));
            }
        }
    }
    Ok(parts.join("/"))
}

pub fn join_inside(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let normalized = normalize_relative(Path::new(relative))?;
    Ok(root.join(normalized))
}

pub fn repository_file(root: &Path, path: &Path) -> Result<PathBuf, String> {
    let root = fs::canonicalize(root)
        .map_err(|error| format!("open repository {}: {error}", root.display()))?;
    let relative = normalize_relative(path)?;
    if relative.is_empty() {
        return Err("repository file path may not be empty".into());
    }
    let candidate = root.join(relative);
    let parent = candidate
        .parent()
        .ok_or_else(|| format!("repository file has no parent: {}", candidate.display()))?;
    let parent = fs::canonicalize(parent)
        .map_err(|error| format!("open repository directory {}: {error}", parent.display()))?;
    if !parent.starts_with(&root) {
        return Err(format!(
            "repository file escapes through its parent: {}",
            path.display()
        ));
    }
    let name = candidate
        .file_name()
        .ok_or_else(|| format!("repository file has no name: {}", path.display()))?;
    Ok(parent.join(name))
}

pub fn glob_matches(pattern: &str, path: &str) -> bool {
    let pattern = pattern.trim_matches('/');
    let path = path.trim_matches('/');
    let patterns = pattern.split('/').collect::<Vec<_>>();
    let parts = path.split('/').collect::<Vec<_>>();
    matches_segments(&patterns, &parts)
}

fn matches_segments(pattern: &[&str], path: &[&str]) -> bool {
    let Some((head, tail)) = pattern.split_first() else {
        return path.is_empty();
    };
    if *head == "**" {
        return matches_segments(tail, path)
            || path
                .split_first()
                .is_some_and(|(_, rest)| matches_segments(pattern, rest));
    }
    path.split_first()
        .is_some_and(|(part, rest)| matches_component(head, part) && matches_segments(tail, rest))
}

fn matches_component(pattern: &str, value: &str) -> bool {
    let pattern = pattern.as_bytes();
    let value = value.as_bytes();
    let mut previous = vec![false; value.len() + 1];
    previous[0] = true;
    for token in pattern {
        let mut current = vec![false; value.len() + 1];
        match token {
            b'*' => {
                current[0] = previous[0];
                for index in 1..=value.len() {
                    current[index] = previous[index] || current[index - 1];
                }
            }
            b'?' => {
                current[1..].copy_from_slice(&previous[..value.len()]);
            }
            literal => {
                for index in 1..=value.len() {
                    current[index] = previous[index - 1] && value[index - 1] == *literal;
                }
            }
        }
        previous = current;
    }
    previous[value.len()]
}

#[cfg(test)]
#[path = "path_test.rs"]
mod path_test;
