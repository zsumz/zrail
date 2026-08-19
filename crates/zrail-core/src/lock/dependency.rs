//! Source-aware dependency identity and stable human-readable evidence.

use serde::{Deserialize, Serialize};

use super::{LockedDependency, LockedDependencyKind, LockedDependencyScope};

/// Complete Cargo source identity retained for one resolved dependency.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum LockedDependencySource {
    /// Dependency resolved to an observed workspace member.
    WorkspaceMember {
        /// Normalized repository-relative member directory.
        directory: String,
        /// Manifest version requirement, when one was declared.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        requirement: Option<String>,
    },
    /// Dependency resolved through a repository-local path outside the workspace.
    RepositoryPath {
        /// Normalized repository-relative dependency directory.
        path: String,
        /// Manifest version requirement, when one was declared.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        requirement: Option<String>,
    },
    /// External dependency resolved from a Cargo registry.
    Registry {
        /// Named registry, or `None` for crates.io unless `index` is present.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        registry: Option<String>,
        /// Explicit registry index URL, mutually exclusive with `registry`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        index: Option<String>,
        /// Non-empty registry version requirement.
        requirement: String,
    },
    /// External dependency resolved from a Git repository.
    Git {
        /// Non-empty repository URL or Cargo Git source string.
        repository: String,
        /// Selected branch; mutually exclusive with `tag` and `rev`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
        /// Selected tag; mutually exclusive with `branch` and `rev`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
        /// Selected revision; mutually exclusive with `branch` and `tag`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rev: Option<String>,
        /// Manifest version requirement, when one was declared.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        requirement: Option<String>,
    },
}

impl LockedDependency {
    /// Renders deterministic human-readable evidence for this dependency.
    ///
    /// Absent legacy fields receive explicit fallback labels. This method does
    /// not validate or canonicalize the dependency first.
    pub fn label(&self) -> String {
        let alias = self.alias.as_deref().unwrap_or(&self.name);
        let crate_root = self.crate_root.as_deref().unwrap_or("unresolved-root");
        let target = self.target.as_deref().unwrap_or("all-targets");
        let optional = self.optional.unwrap_or(false);
        let default_features = self.default_features.unwrap_or(true);
        let features = self.features.join(",");
        let source = self
            .source
            .as_ref()
            .map_or_else(|| "legacy".into(), LockedDependencySource::label);
        format!(
            "{}:{}:{target}:{alias}[{crate_root}]=>{}:{source}:optional={optional}:default-features={default_features}:features=[{features}]",
            self.scope.label(),
            self.kind.label(),
            self.name,
        )
    }
}

impl LockedDependencySource {
    fn label(&self) -> String {
        match self {
            Self::WorkspaceMember {
                directory,
                requirement,
            } => format!(
                "workspace:{directory}:version={}",
                requirement.as_deref().unwrap_or("")
            ),
            Self::RepositoryPath { path, requirement } => format!(
                "path:{path}:version={}",
                requirement.as_deref().unwrap_or("")
            ),
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

impl LockedDependencyKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Development => "development",
            Self::Build => "build",
        }
    }
}

impl LockedDependencyScope {
    const fn label(self) -> &'static str {
        match self {
            Self::Internal => "internal",
            Self::External => "external",
        }
    }
}
