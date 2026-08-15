//! Source-aware dependency identity and stable human-readable evidence.

use serde::{Deserialize, Serialize};

use super::{LockedDependency, LockedDependencyKind, LockedDependencyScope};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum LockedDependencySource {
    WorkspaceMember {
        directory: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        requirement: Option<String>,
    },
    RepositoryPath {
        path: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        requirement: Option<String>,
    },
    Registry {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        registry: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        index: Option<String>,
        requirement: String,
    },
    Git {
        repository: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rev: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        requirement: Option<String>,
    },
}

impl LockedDependency {
    pub fn label(&self) -> String {
        let alias = self.alias.as_deref().unwrap_or(&self.name);
        let target = self.target.as_deref().unwrap_or("all-targets");
        let optional = self.optional.unwrap_or(false);
        let default_features = self.default_features.unwrap_or(true);
        let features = self.features.join(",");
        let source = self
            .source
            .as_ref()
            .map_or_else(|| "legacy".into(), LockedDependencySource::label);
        format!(
            "{}:{}:{target}:{alias}=>{}:{source}:optional={optional}:default-features={default_features}:features=[{features}]",
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
