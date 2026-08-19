//! Dependency-policy schema, including reviewed external crate-root attestations.

use serde::{Deserialize, Serialize};

use crate::contract::{CycleMode, DependencyMode, PolicyMode};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Dependency-topology policy for the repository.
pub struct DependenciesContract {
    /// Selects whether topology is lock-authoritative or directly observed.
    pub mode: DependencyMode,
    /// Governs packages that match no declared architecture layer.
    pub unassigned_packages: PolicyMode,
    /// Governs cycles in the package dependency graph.
    pub cycles: CycleMode,
    #[serde(default, rename = "crate_root")]
    /// Reviewed source-root overrides for external packages.
    pub crate_roots: Vec<CrateRootContract>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// A reviewed mapping from an external package to its analyzable source root.
pub struct CrateRootContract {
    /// Cargo package name to which the mapping applies.
    pub package: String,
    /// Repository-relative path within the resolved package source.
    pub root: String,
    /// Human justification for trusting this nonstandard root.
    pub reason: String,
    #[serde(default)]
    /// Provenance that distinguishes packages sharing the same name.
    pub source: CrateRootSource,
}

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
/// Provenance identity for a reviewed external crate-root mapping.
pub enum CrateRootSource {
    #[default]
    /// Legacy name-only authority, retained for older contracts.
    Legacy,
    /// A package resolved from a Cargo registry.
    Registry {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Registry name, or crates.io when absent.
        registry: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Registry index URL, or that registry's default index when absent.
        index: Option<String>,
        /// Cargo version requirement identifying the reviewed package line.
        requirement: String,
    },
    /// A package resolved from a Git repository.
    Git {
        /// Canonical repository URL recorded by Cargo.
        repository: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Requested branch, when resolution was branch-based.
        branch: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Requested tag, when resolution was tag-based.
        tag: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Requested revision, when explicitly pinned.
        rev: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Optional Cargo version requirement applied to the Git package.
        requirement: Option<String>,
    },
}

impl CrateRootSource {
    /// Returns a deterministic string containing every provenance discriminator.
    ///
    /// The string is intended for equality subjects and diagnostics, not parsing.
    pub fn identity(&self) -> String {
        match self {
            Self::Legacy => "legacy-name-only".into(),
            Self::Registry {
                registry,
                index,
                requirement,
            } => format!(
                "registry:{}:{}:{requirement}",
                registry.as_deref().unwrap_or("crates-io"),
                index.as_deref().unwrap_or("default-index")
            ),
            Self::Git {
                repository,
                branch,
                tag,
                rev,
                requirement,
            } => format!(
                "git:{repository}:branch={}:tag={}:rev={}:version={}",
                branch.as_deref().unwrap_or(""),
                tag.as_deref().unwrap_or(""),
                rev.as_deref().unwrap_or(""),
                requirement.as_deref().unwrap_or("")
            ),
        }
    }
}
