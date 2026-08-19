//! Validation and ordering for source-aware locked dependencies.

use super::super::{LockError, LockedDependency, LockedDependencyScope, LockedDependencySource};
use super::valid_root;

pub(super) fn canonicalize(dependency: &mut LockedDependency) -> Result<(), LockError> {
    dependency.features.sort();
    if dependency
        .features
        .windows(2)
        .any(|pair| pair[0] == pair[1])
    {
        return Err(LockError(format!(
            "dependency {} has duplicate features",
            dependency.name
        )));
    }
    if dependency.features.iter().any(|feature| !nonempty(feature)) {
        return Err(LockError(format!(
            "dependency {} has an empty feature",
            dependency.name
        )));
    }
    validate_complete(dependency)?;
    validate_source(dependency)
}

fn validate_complete(dependency: &LockedDependency) -> Result<(), LockError> {
    let alias = dependency
        .alias
        .as_deref()
        .ok_or_else(|| LockError(format!("dependency {} requires an alias", dependency.name)))?;
    if !nonempty(alias) || dependency.optional.is_none() || dependency.default_features.is_none() {
        return Err(LockError(format!(
            "dependency {} requires non-empty alias, optional, and default_features state",
            dependency.name
        )));
    }
    if dependency
        .target
        .as_ref()
        .is_some_and(|target| !nonempty(target))
    {
        return Err(LockError(format!(
            "dependency {} target selector may not be empty",
            dependency.name
        )));
    }
    if dependency.source.is_none() {
        return Err(LockError(format!(
            "dependency {} requires source identity",
            dependency.name
        )));
    }
    if dependency
        .crate_root
        .as_deref()
        .is_some_and(|root| !valid_crate_root(root))
    {
        return Err(LockError(format!(
            "dependency {} has an invalid effective crate root",
            dependency.name
        )));
    }
    if dependency.scope == LockedDependencyScope::Internal && dependency.crate_root.is_none() {
        return Err(LockError(format!(
            "internal dependency {} requires an effective crate root",
            dependency.name
        )));
    }
    Ok(())
}

fn validate_source(dependency: &LockedDependency) -> Result<(), LockError> {
    let source = dependency
        .source
        .as_ref()
        .ok_or_else(|| LockError(format!("dependency {} has no source", dependency.name)))?;
    let valid = match source {
        LockedDependencySource::WorkspaceMember {
            directory,
            requirement,
        } => {
            dependency.scope == LockedDependencyScope::Internal
                && valid_root(directory)
                && optional_nonempty(requirement.as_ref())
        }
        LockedDependencySource::RepositoryPath { path, requirement } => {
            dependency.scope == LockedDependencyScope::External
                && valid_root(path)
                && optional_nonempty(requirement.as_ref())
        }
        LockedDependencySource::Registry {
            registry,
            index,
            requirement,
        } => {
            dependency.scope == LockedDependencyScope::External
                && !(registry.is_some() && index.is_some())
                && nonempty(requirement)
                && optional_nonempty(registry.as_ref())
                && optional_nonempty(index.as_ref())
        }
        LockedDependencySource::Git {
            repository,
            branch,
            tag,
            rev,
            requirement,
        } => {
            let references = [branch, tag, rev]
                .into_iter()
                .filter(|reference| reference.is_some())
                .count();
            dependency.scope == LockedDependencyScope::External
                && nonempty(repository)
                && references <= 1
                && optional_nonempty(branch.as_ref())
                && optional_nonempty(tag.as_ref())
                && optional_nonempty(rev.as_ref())
                && optional_nonempty(requirement.as_ref())
        }
    };
    if valid {
        Ok(())
    } else {
        Err(LockError(format!(
            "dependency {} has invalid source identity",
            dependency.name
        )))
    }
}

fn nonempty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_crate_root(root: &str) -> bool {
    if root.starts_with("r#") || matches!(root, "_" | "Self" | "crate" | "self" | "super") {
        return false;
    }
    let mut bytes = root.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn optional_nonempty(value: Option<&String>) -> bool {
    value.map(String::as_str).is_none_or(nonempty)
}
