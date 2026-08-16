//! Dependency-policy schema, including reviewed external crate-root attestations.

use serde::{Deserialize, Serialize};

use crate::contract::{CycleMode, DependencyMode, PolicyMode};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DependenciesContract {
    pub mode: DependencyMode,
    pub unassigned_packages: PolicyMode,
    pub cycles: CycleMode,
    #[serde(default, rename = "crate_root")]
    pub crate_roots: Vec<CrateRootContract>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CrateRootContract {
    pub package: String,
    pub root: String,
    pub reason: String,
    #[serde(default)]
    pub source: CrateRootSource,
}

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum CrateRootSource {
    #[default]
    Legacy,
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

impl CrateRootSource {
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
