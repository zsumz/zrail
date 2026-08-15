//! Normalized Cargo packages and dependency edges.

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum DependencyKind {
    Normal,
    Development,
    Build,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Dependency {
    pub(crate) name: String,
    pub(crate) kind: DependencyKind,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct DependencyPath {
    pub(crate) path: String,
    pub(crate) workspace_relative: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Package {
    pub(crate) name: String,
    pub(crate) directory: String,
    pub(crate) dependencies: Vec<Dependency>,
    pub(crate) dependency_paths: Vec<DependencyPath>,
    pub(crate) targets: Vec<String>,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CargoWorkspace {
    pub(crate) declared_members: Vec<String>,
    pub(crate) observed_members: Vec<String>,
    pub(crate) packages: Vec<Package>,
}
