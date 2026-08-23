//! Macro expansion authority follows resolved compiler, repository, or dependency origin.

use std::collections::BTreeSet;

use zrail_core::AnalysisQuality;

use crate::cargo::{CrateRootAuthority, DependencySource, Package, rust_crate_root};

use super::{MacroCandidate, MacroDerivation, MacroExpansionFact, MacroOrigin};

const MAX_MACRO_ORIGINS: usize = 4;

pub(super) fn resolve(expansion: &mut MacroExpansionFact, packages: &[&Package]) {
    for candidate in &mut expansion.candidates {
        resolve_candidate(candidate, packages);
    }
    expansion.refresh_quality();
}

fn resolve_candidate(candidate: &mut MacroCandidate, packages: &[&Package]) {
    let local_module = candidate
        .origins
        .iter()
        .any(|origin| matches!(origin, MacroOrigin::Pending { local_module: true }));
    if !candidate
        .origins
        .iter()
        .any(|origin| matches!(origin, MacroOrigin::Pending { .. }))
    {
        return;
    }
    if candidate.observation.quality == AnalysisQuality::Unresolved {
        if local_module && !compiler_builtin(&candidate.observation.name) {
            candidate.observation.quality = AnalysisQuality::Conservative;
            candidate.origins = repository_origins(packages);
        } else {
            candidate.origins = vec![MacroOrigin::Unresolved];
        }
        return;
    }
    let root = candidate
        .observation
        .name
        .split("::")
        .next()
        .map(visible_root)
        .unwrap_or_default();
    let dependencies = dependency_origins(root, packages);
    let own_package = packages
        .iter()
        .any(|package| rust_crate_root(&package.name) == root);
    if !dependencies.is_empty() && candidate.derivation == MacroDerivation::Written {
        candidate.derivation = MacroDerivation::DependencyRoot;
    }
    candidate.origins = if dependencies.len() > MAX_MACRO_ORIGINS {
        vec![MacroOrigin::Unresolved]
    } else if !dependencies.is_empty() && !local_module {
        dependencies
    } else if local_module || own_package || matches!(root, "crate" | "self" | "super" | "Self") {
        repository_origins(packages)
    } else if compiler_builtin(&candidate.observation.name) {
        vec![MacroOrigin::CompilerBuiltin]
    } else {
        vec![MacroOrigin::Unresolved]
    };
}

fn dependency_origins(root: &str, packages: &[&Package]) -> Vec<MacroOrigin> {
    packages
        .iter()
        .flat_map(|package| &package.dependencies)
        .filter(|dependency| rust_crate_root(&dependency.crate_root) == root)
        .map(|dependency| {
            if dependency.crate_root_authority == CrateRootAuthority::Unresolved {
                return MacroOrigin::Unresolved;
            }
            match &dependency.source {
                DependencySource::WorkspaceMember { directory, .. } => MacroOrigin::Repository {
                    package: dependency.name.clone(),
                    directory: directory.clone(),
                },
                DependencySource::RepositoryPath { path, .. } => MacroOrigin::Repository {
                    package: dependency.name.clone(),
                    directory: path.clone(),
                },
                source @ (DependencySource::Registry { .. } | DependencySource::Git { .. }) => {
                    MacroOrigin::External {
                        package: dependency.name.clone(),
                        source: source.clone(),
                    }
                }
            }
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn repository_origins(packages: &[&Package]) -> Vec<MacroOrigin> {
    let origins = packages
        .iter()
        .map(|package| MacroOrigin::Repository {
            package: package.name.clone(),
            directory: package.directory.clone(),
        })
        .collect::<BTreeSet<_>>();
    if origins.is_empty() || origins.len() > MAX_MACRO_ORIGINS {
        vec![MacroOrigin::Unresolved]
    } else {
        origins.into_iter().collect()
    }
}

fn compiler_builtin(name: &str) -> bool {
    let mut segments = name.split("::");
    let root = segments.next().unwrap_or_default();
    let leaf = name.rsplit("::").next().unwrap_or(name);
    if name.contains("::") && !matches!(visible_root(root), "alloc" | "core" | "std") {
        return false;
    }
    matches!(
        leaf,
        "Clone"
            | "Copy"
            | "Debug"
            | "Default"
            | "Eq"
            | "Hash"
            | "Ord"
            | "PartialEq"
            | "PartialOrd"
            | "assert"
            | "assert_eq"
            | "assert_ne"
            | "asm"
            | "addr_of"
            | "addr_of_mut"
            | "cfg"
            | "column"
            | "compile_error"
            | "concat"
            | "concat_bytes"
            | "dbg"
            | "debug_assert"
            | "debug_assert_eq"
            | "debug_assert_ne"
            | "eprint"
            | "eprintln"
            | "env"
            | "file"
            | "format"
            | "format_args"
            | "global_asm"
            | "include"
            | "include_bytes"
            | "include_str"
            | "line"
            | "matches"
            | "module_path"
            | "naked_asm"
            | "offset_of"
            | "option_env"
            | "panic"
            | "print"
            | "println"
            | "stringify"
            | "thread_local"
            | "todo"
            | "unimplemented"
            | "unreachable"
            | "vec"
            | "write"
            | "writeln"
    )
}

fn visible_root(root: &str) -> &str {
    root.strip_prefix("r#").unwrap_or(root)
}

#[cfg(test)]
#[path = "macro_origins_test.rs"]
mod macro_origins_test;
