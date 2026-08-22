//! Normalized Cargo packages and dependency edges.

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum DependencyKind {
    Normal,
    Development,
    Build,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Dependency {
    pub(crate) alias: String,
    pub(crate) name: String,
    pub(crate) explicit_package: bool,
    pub(crate) crate_root: String,
    pub(crate) crate_root_authority: CrateRootAuthority,
    pub(crate) kind: DependencyKind,
    pub(crate) target: Option<String>,
    pub(crate) optional: bool,
    pub(crate) default_features: bool,
    pub(crate) features: Vec<String>,
    pub(crate) source: DependencySource,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CrateRootAuthority {
    DeclaredAlias,
    InspectedLibrary,
    Attested,
    Unresolved,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum DependencySource {
    WorkspaceMember {
        directory: String,
        requirement: Option<String>,
    },
    RepositoryPath {
        path: String,
        requirement: Option<String>,
    },
    Registry {
        registry: Option<String>,
        index: Option<String>,
        requirement: String,
    },
    Git {
        repository: String,
        branch: Option<String>,
        tag: Option<String>,
        rev: Option<String>,
        requirement: Option<String>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Package {
    pub(crate) name: String,
    pub(crate) directory: String,
    pub(crate) dependencies: Vec<Dependency>,
    pub(crate) targets: Vec<CargoTarget>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct CargoTarget {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) kind: CargoTargetKind,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CargoTargetKind {
    Library,
    Binary,
    Example,
    Test,
    Benchmark,
    BuildScript,
}

impl Package {
    pub(crate) fn contains_file(&self, file: &str) -> bool {
        self.directory == "."
            || file == self.directory
            || file.starts_with(&format!("{}/", self.directory))
    }

    pub(crate) fn manifest_path(&self) -> String {
        if self.directory == "." {
            "Cargo.toml".into()
        } else {
            format!("{}/Cargo.toml", self.directory)
        }
    }

    pub(crate) fn library_crate_root(&self) -> Option<&str> {
        self.targets
            .iter()
            .find(|target| target.kind == CargoTargetKind::Library)
            .map(|target| target.name.as_str())
    }
}

pub(crate) fn rust_crate_root(name: &str) -> String {
    name.replace('-', "_")
}

impl Dependency {
    pub(crate) fn internal_package(&self) -> Option<&str> {
        matches!(self.source, DependencySource::WorkspaceMember { .. })
            .then_some(self.name.as_str())
    }

    pub(crate) fn repository_path(&self) -> Option<&str> {
        match &self.source {
            DependencySource::RepositoryPath { path, .. } => Some(path),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CargoWorkspace {
    pub(crate) declared_members: Vec<String>,
    pub(crate) observed_members: Vec<String>,
    pub(crate) packages: Vec<Package>,
    pub(crate) authority_surfaces: Vec<CargoAuthoritySurface>,
    pub(crate) manifest_scopes: BTreeMap<String, ManifestScope>,
}

impl CargoWorkspace {
    pub(crate) fn source_is_active(&self, path: &str) -> bool {
        self.manifest_scopes
            .iter()
            .filter(|(directory, _)| contains_path(directory, path))
            .max_by_key(|(directory, _)| directory_depth(directory))
            .is_none_or(|(_, scope)| *scope == ManifestScope::Active)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ManifestScope {
    Active,
    Ignored,
}

fn directory_depth(directory: &str) -> usize {
    usize::from(directory != ".") + directory.matches('/').count()
}

fn contains_path(directory: &str, path: &str) -> bool {
    directory == "." || path == directory || path.starts_with(&format!("{directory}/"))
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CargoAuthorityKind {
    Resolution,
    RepositoryConfiguration,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct CargoAuthoritySurface {
    pub(crate) kind: CargoAuthorityKind,
    pub(crate) path: String,
    pub(crate) surface: String,
}
