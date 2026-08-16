//! Strict source-aware projection of one Cargo dependency specification.

mod fields;

use std::collections::BTreeMap;

use toml::Value;

use super::{model::DependencySource, workspace::resolve_inside};
use fields::{default_features, features, nonempty, optional_bool, optional_string, validate_keys};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct DependencySpec {
    pub(super) name: String,
    pub(super) explicit_package: bool,
    pub(super) optional: bool,
    pub(super) default_features: bool,
    pub(super) features: Vec<String>,
    pub(super) source: DependencySource,
}

pub(super) type WorkspaceDependencies = BTreeMap<String, DependencySpec>;

pub(super) fn parse(
    alias: &str,
    value: &Value,
    base: &str,
    workspace: Option<&WorkspaceDependencies>,
) -> Result<DependencySpec, String> {
    if let Some(requirement) = value.as_str() {
        return Ok(DependencySpec {
            name: alias.into(),
            explicit_package: false,
            optional: false,
            default_features: true,
            features: Vec::new(),
            source: registry_source(None, None, requirement)?,
        });
    }
    let table = value
        .as_table()
        .ok_or_else(|| "specification must be a version string or table".to_owned())?;
    validate_keys(table)?;
    if optional_bool(table, "workspace")? == Some(true) {
        return inherit(alias, table, workspace);
    }
    if optional_bool(table, "workspace")? == Some(false) {
        return Err("workspace inheritance must be true when present".into());
    }
    let package = optional_string(table, "package")?;
    let explicit_package = package.is_some();
    let name = package.unwrap_or_else(|| alias.into());
    let optional = optional_bool(table, "optional")?.unwrap_or(false);
    let default_features = default_features(table)?.unwrap_or(true);
    let features = features(table)?;
    let source = source(table, base)?;
    Ok(DependencySpec {
        name,
        explicit_package,
        optional,
        default_features,
        features,
        source,
    })
}

fn inherit(
    alias: &str,
    table: &toml::map::Map<String, Value>,
    workspace: Option<&WorkspaceDependencies>,
) -> Result<DependencySpec, String> {
    for forbidden in [
        "package",
        "path",
        "version",
        "git",
        "registry",
        "registry-index",
        "branch",
        "tag",
        "rev",
        "default-features",
        "default_features",
    ] {
        if table.contains_key(forbidden) {
            return Err(format!(
                "workspace inheritance may not override {forbidden}"
            ));
        }
    }
    let mut inherited = workspace
        .and_then(|values| values.get(alias))
        .cloned()
        .ok_or_else(|| "workspace dependency is not declared at the workspace root".to_owned())?;
    if let Some(optional) = optional_bool(table, "optional")? {
        inherited.optional = optional;
    }
    inherited.features.extend(features(table)?);
    inherited.features.sort();
    inherited.features.dedup();
    Ok(inherited)
}

fn source(table: &toml::map::Map<String, Value>, base: &str) -> Result<DependencySource, String> {
    let path = optional_string(table, "path")?;
    let git = optional_string(table, "git")?;
    let registry = optional_string(table, "registry")?;
    let index = optional_string(table, "registry-index")?;
    let requirement = optional_string(table, "version")?;
    let sources = [
        path.is_some(),
        git.is_some(),
        registry.is_some() || index.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count();
    if sources > 1 {
        return Err("path, git, and registry sources are mutually exclusive".into());
    }
    if let Some(path) = path {
        reject_git_reference(table)?;
        let path = resolve_inside(base, &path)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "path dependency resolves outside the repository".to_owned())?;
        return Ok(DependencySource::RepositoryPath { path, requirement });
    }
    if let Some(repository) = git {
        let reference = git_reference(table)?;
        return Ok(DependencySource::Git {
            repository: nonempty(repository, "git")?,
            branch: reference.branch,
            tag: reference.tag,
            rev: reference.rev,
            requirement,
        });
    }
    reject_git_reference(table)?;
    let requirement =
        requirement.ok_or_else(|| "registry dependency requires a version".to_owned())?;
    registry_source(registry, index, &requirement)
}

fn registry_source(
    registry: Option<String>,
    index: Option<String>,
    requirement: &str,
) -> Result<DependencySource, String> {
    if registry.is_some() && index.is_some() {
        return Err("registry and registry-index are mutually exclusive".into());
    }
    Ok(DependencySource::Registry {
        registry: registry
            .map(|value| nonempty(value, "registry"))
            .transpose()?,
        index: index
            .map(|value| nonempty(value, "registry-index"))
            .transpose()?,
        requirement: nonempty(requirement.into(), "version")?,
    })
}

fn git_reference(table: &toml::map::Map<String, Value>) -> Result<GitReference, String> {
    let branch = optional_string(table, "branch")?;
    let tag = optional_string(table, "tag")?;
    let rev = optional_string(table, "rev")?;
    let references = [branch.is_some(), tag.is_some(), rev.is_some()]
        .into_iter()
        .filter(|present| *present)
        .count();
    if references > 1 {
        return Err("git branch, tag, and rev are mutually exclusive".into());
    }
    Ok(GitReference { branch, tag, rev })
}

fn reject_git_reference(table: &toml::map::Map<String, Value>) -> Result<(), String> {
    let reference = git_reference(table)?;
    if reference.branch.is_some() || reference.tag.is_some() || reference.rev.is_some() {
        return Err("git branch, tag, or rev requires a git source".into());
    }
    Ok(())
}

struct GitReference {
    branch: Option<String>,
    tag: Option<String>,
    rev: Option<String>,
}
