//! Normalized Cargo packages and dependency edges.

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
    pub(crate) kind: DependencyKind,
    pub(crate) target: Option<String>,
    pub(crate) optional: bool,
    pub(crate) default_features: bool,
    pub(crate) features: Vec<String>,
    pub(crate) source: DependencySource,
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
    pub(crate) resolution_overrides: Vec<CargoResolutionOverride>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct CargoResolutionOverride {
    pub(crate) path: String,
    pub(crate) surface: String,
}
