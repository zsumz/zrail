//! Lexical Rust module paths stay normalized and bounded by the repository.

use std::path::{Component, Path};

use super::model::ModuleDeclaration;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ModuleTarget {
    Exact(String),
    Search { direct: String, nested: String },
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum SubmoduleBase {
    SourceParent,
    FileStemDirectory,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ResolutionError {
    Escape(String),
    Unresolved(String),
}

impl ResolutionError {
    pub(crate) fn message(&self) -> &str {
        match self {
            Self::Escape(message) | Self::Unresolved(message) => message,
        }
    }
}

pub(crate) fn join_relative(base: &str, relative: &str) -> Result<String, ResolutionError> {
    if relative.contains('\\') {
        return Err(ResolutionError::Escape(format!(
            "path uses a platform-dependent separator: {relative:?}"
        )));
    }
    let mut parts = normalized_parts(base)?;
    for component in Path::new(relative).components() {
        match component {
            Component::CurDir => {}
            Component::Normal(value) => parts.push(value.to_string_lossy().into_owned()),
            Component::ParentDir if parts.pop().is_some() => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(ResolutionError::Escape(format!(
                    "path escapes the repository: {relative:?}"
                )));
            }
        }
    }
    if parts.is_empty() {
        return Err(ResolutionError::Escape(format!(
            "path does not name a repository file: {relative:?}"
        )));
    }
    Ok(parts.join("/"))
}

pub(crate) fn module_target(
    source: &str,
    submodule_base: SubmoduleBase,
    declaration: &ModuleDeclaration,
) -> Result<ModuleTarget, ResolutionError> {
    if declaration.unresolved_path {
        return Err(ResolutionError::Unresolved(format!(
            "module {} has a conditional or malformed path attribute",
            declaration.name
        )));
    }
    let source_parent = parent(source);
    if declaration.inline_ancestors.is_empty() {
        return declaration.path.as_deref().map_or_else(
            || {
                search_target(
                    &module_directory(source, submodule_base)?,
                    &declaration.name,
                )
            },
            |path| join_relative(&source_parent, path).map(ModuleTarget::Exact),
        );
    }
    let mut base = module_directory(source, submodule_base)?;
    for (index, ancestor) in declaration.inline_ancestors.iter().enumerate() {
        if ancestor.unresolved_path {
            return Err(ResolutionError::Unresolved(format!(
                "inline module {} has a conditional or malformed path attribute",
                ancestor.name
            )));
        }
        base = if let Some(path) = &ancestor.path {
            let attribute_base = if index == 0 { &source_parent } else { &base };
            join_relative(attribute_base, path)?
        } else {
            join_relative(&base, &ancestor.name)?
        };
    }
    declaration.path.as_deref().map_or_else(
        || search_target(&base, &declaration.name),
        |path| join_relative(&base, path).map(ModuleTarget::Exact),
    )
}

pub(crate) fn parent(path: &str) -> String {
    Path::new(path)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map_or_else(|| ".".to_owned(), |parent| parent.to_string_lossy().into())
}

fn module_directory(
    source: &str,
    submodule_base: SubmoduleBase,
) -> Result<String, ResolutionError> {
    let path = Path::new(source);
    let parent = parent(source);
    match submodule_base {
        SubmoduleBase::SourceParent => Ok(parent),
        SubmoduleBase::FileStemDirectory => {
            let stem = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .ok_or_else(|| {
                    ResolutionError::Unresolved(format!(
                        "Rust source has no UTF-8 file stem: {source:?}"
                    ))
                })?;
            join_relative(&parent, stem)
        }
    }
}

fn search_target(base: &str, name: &str) -> Result<ModuleTarget, ResolutionError> {
    Ok(ModuleTarget::Search {
        direct: join_relative(base, &format!("{name}.rs"))?,
        nested: join_relative(base, &format!("{name}/mod.rs"))?,
    })
}

fn normalized_parts(path: &str) -> Result<Vec<String>, ResolutionError> {
    if path == "." {
        return Ok(Vec::new());
    }
    let mut parts = Vec::new();
    for component in Path::new(path).components() {
        match component {
            Component::Normal(value) => parts.push(value.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(ResolutionError::Escape(format!(
                    "base path is not repository-relative: {path:?}"
                )));
            }
        }
    }
    Ok(parts)
}

#[cfg(test)]
#[path = "paths_test.rs"]
mod paths_test;
