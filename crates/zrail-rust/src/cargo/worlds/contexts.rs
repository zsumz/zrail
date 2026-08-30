//! Split Cargo contexts propagate through every downstream internal dependency.

use std::collections::{BTreeMap, VecDeque, btree_map::Entry};

use crate::cargo::{CargoWorkspace, Dependency, DependencyKind, Package};

pub(super) struct SplitContexts {
    causes: BTreeMap<String, SplitCause>,
}

#[derive(Clone)]
enum SplitCause {
    ProcMacro,
    Direct(EdgeWitness),
    Transitive(EdgeWitness),
}

#[derive(Clone)]
struct EdgeWitness {
    source: String,
    destination: String,
    alias: String,
    kind: DependencyKind,
    target: Option<String>,
}

impl SplitContexts {
    pub(super) fn new(cargo: &CargoWorkspace) -> Self {
        let mut causes = BTreeMap::new();
        for package in &cargo.packages {
            if package.is_proc_macro() {
                causes.insert(package.name.clone(), SplitCause::ProcMacro);
            }
        }
        for package in &cargo.packages {
            for dependency in &package.dependencies {
                if directly_split(dependency) {
                    seed_dependency(&mut causes, package, dependency);
                }
            }
        }
        propagate(cargo, &mut causes);
        Self { causes }
    }

    pub(super) fn edge_is_split(&self, source: &Package, dependency: &Dependency) -> bool {
        directly_split(dependency) || self.causes.contains_key(&source.name)
    }

    pub(super) fn source_context(&self, source: &Package) -> Option<String> {
        self.witness(&source.name)
    }

    pub(super) fn packages(&self) -> impl Iterator<Item = &str> {
        self.causes.keys().map(String::as_str)
    }

    pub(super) fn witness(&self, package: &str) -> Option<String> {
        self.causes
            .contains_key(package)
            .then(|| self.describe(package))
    }

    fn describe(&self, package: &str) -> String {
        let mut current = package;
        let mut explanation = format!("package {package:?} is context-split because ");
        for remaining in (1..=self.causes.len()).rev() {
            match &self.causes[current] {
                SplitCause::ProcMacro => {
                    explanation.push_str("it is a Cargo proc-macro host target");
                    return explanation;
                }
                SplitCause::Direct(edge) => {
                    explanation.push_str("it is the destination of ");
                    explanation.push_str(&edge.description());
                    return explanation;
                }
                SplitCause::Transitive(edge) => {
                    explanation.push_str("it is reached through ");
                    explanation.push_str(&edge.description());
                    explanation.push_str("; ");
                    current = &edge.source;
                }
            }
            if remaining == 1 {
                break;
            }
        }
        explanation.push_str("its bounded context witness is cyclic");
        explanation
    }
}

impl EdgeWitness {
    fn new(source: &Package, dependency: &Dependency, destination: &str) -> Self {
        Self {
            source: source.name.clone(),
            destination: destination.into(),
            alias: dependency.alias.clone(),
            kind: dependency.kind,
            target: dependency.target.clone(),
        }
    }

    fn description(&self) -> String {
        format!(
            "the {} dependency edge from package {:?} to package {:?} (alias {:?}, target condition {})",
            kind_name(self.kind),
            self.source,
            self.destination,
            self.alias,
            target_name(self.target.as_deref())
        )
    }
}

fn seed_dependency(
    causes: &mut BTreeMap<String, SplitCause>,
    source: &Package,
    dependency: &Dependency,
) {
    let Some(destination) = dependency.internal_package() else {
        return;
    };
    causes
        .entry(destination.into())
        .or_insert_with(|| SplitCause::Direct(EdgeWitness::new(source, dependency, destination)));
}

fn propagate(cargo: &CargoWorkspace, causes: &mut BTreeMap<String, SplitCause>) {
    let packages = cargo
        .packages
        .iter()
        .map(|package| (package.name.as_str(), package))
        .collect::<BTreeMap<_, _>>();
    let mut queue = causes.keys().cloned().collect::<VecDeque<_>>();
    while let Some(source_name) = queue.pop_front() {
        let Some(source) = packages.get(source_name.as_str()).copied() else {
            continue;
        };
        for dependency in &source.dependencies {
            let Some(destination) = dependency.internal_package() else {
                continue;
            };
            if let Entry::Vacant(entry) = causes.entry(destination.into()) {
                entry.insert(SplitCause::Transitive(EdgeWitness::new(
                    source,
                    dependency,
                    destination,
                )));
                queue.push_back(destination.into());
            }
        }
    }
}

fn directly_split(dependency: &Dependency) -> bool {
    dependency.target.is_some() || dependency.kind != DependencyKind::Normal
}

pub(super) const fn kind_name(kind: DependencyKind) -> &'static str {
    match kind {
        DependencyKind::Normal => "normal",
        DependencyKind::Development => "development",
        DependencyKind::Build => "build",
    }
}

pub(super) fn target_name(target: Option<&str>) -> String {
    target.map_or_else(|| "<all targets>".into(), |target| format!("{target:?}"))
}
