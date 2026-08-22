//! Cargo roots and Rust source edges must form one closed, analyzable graph.

mod boundary;
mod external_module;
mod include;
mod item_macros;

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use zrail_core::{Contract, Finding};

use crate::{
    cargo::{CargoTargetKind, CargoWorkspace},
    inventory::{RepositoryEntryKind, RepositoryInventory},
    source::{
        Reachability, ReachabilityKind, RustFileFacts, SourceIndex, SourceSyntax, SubmoduleBase,
        join_relative,
    },
};

pub(crate) fn analyze(
    contract: &Contract,
    inventory: &RepositoryInventory,
    cargo: &CargoWorkspace,
    source: &SourceIndex,
) -> SourceGraphAnalysis {
    Walker::new(contract, inventory, cargo, source).run()
}

pub(crate) struct SourceGraphAnalysis {
    pub(crate) reachability: BTreeMap<String, Reachability>,
    pub(crate) packages: BTreeMap<String, BTreeSet<String>>,
    pub(crate) findings: Vec<Finding>,
}

pub(crate) fn item_macro_authorities(contract: &Contract, file: &RustFileFacts) -> Vec<usize> {
    item_macros::authorities_for_file(contract, file)
}

pub(crate) fn item_macro_selector(allowance: &zrail_core::ItemMacroContract) -> String {
    item_macros::selector_name(allowance)
}

pub(crate) fn review_item_macros(contract: &Contract, source: &SourceIndex) -> Vec<Finding> {
    item_macros::review(contract, source)
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct TraversalContext {
    reachability: Reachability,
    package: String,
}

impl TraversalContext {
    fn with_test_guard(&self, guarded: bool) -> Self {
        Self {
            reachability: if guarded {
                Reachability::test()
            } else {
                self.reachability
            },
            package: self.package.clone(),
        }
    }
}

struct Walker<'a> {
    contract: &'a Contract,
    inventory: &'a RepositoryInventory,
    cargo: &'a CargoWorkspace,
    findings: Vec<Finding>,
    facts: BTreeMap<&'a str, &'a RustFileFacts>,
    entries: BTreeMap<&'a str, RepositoryEntryKind>,
    reached: BTreeMap<String, Reachability>,
    reached_packages: BTreeMap<String, BTreeSet<String>>,
    seen_out_dir: BTreeSet<(String, String)>,
    reported: BTreeSet<(String, String)>,
    visited: BTreeSet<(String, SubmoduleBase, TraversalContext)>,
    queue: VecDeque<(String, SubmoduleBase, TraversalContext)>,
}

impl<'a> Walker<'a> {
    fn new(
        contract: &'a Contract,
        inventory: &'a RepositoryInventory,
        cargo: &'a CargoWorkspace,
        source: &'a SourceIndex,
    ) -> Self {
        Self {
            contract,
            inventory,
            cargo,
            findings: Vec::new(),
            facts: source
                .files
                .iter()
                .map(|file| (file.relative.as_str(), file))
                .collect(),
            entries: inventory
                .entries
                .iter()
                .map(|entry| (entry.relative.as_str(), entry.kind))
                .collect(),
            reached: BTreeMap::new(),
            reached_packages: BTreeMap::new(),
            seen_out_dir: BTreeSet::new(),
            reported: BTreeSet::new(),
            visited: BTreeSet::new(),
            queue: VecDeque::new(),
        }
    }

    fn run(mut self) -> SourceGraphAnalysis {
        self.seed_cargo_targets();
        while let Some((path, submodule_base, context)) = self.queue.pop_front() {
            self.walk_file(&path, submodule_base, &context);
        }
        self.reject_orphans();
        self.reject_stale_out_dir();
        SourceGraphAnalysis {
            reachability: self.reached,
            packages: self.reached_packages,
            findings: self.findings,
        }
    }

    fn seed_cargo_targets(&mut self) {
        for package in &self.cargo.packages {
            if package.targets.is_empty() {
                let message = format!("Cargo package {:?} has no Rust target", package.name);
                self.missing(&package.manifest_path(), None, message);
            }
            for target in &package.targets {
                let reachability = target_reachability(target.kind);
                match join_relative(&package.directory, &target.path) {
                    Ok(path) => self.follow(
                        &package.manifest_path(),
                        None,
                        path,
                        &format!("Cargo target {:?}", target.path),
                        SubmoduleBase::SourceParent,
                        SourceSyntax::Items,
                        TraversalContext {
                            reachability,
                            package: package.name.clone(),
                        },
                    ),
                    Err(error) => self.resolution_error(
                        &package.manifest_path(),
                        None,
                        &error,
                        &format!("Cargo target {:?}", target.path),
                    ),
                }
            }
        }
    }

    fn walk_file(&mut self, path: &str, submodule_base: SubmoduleBase, context: &TraversalContext) {
        let (modules, includes) = {
            let Some(file) = self.facts.get(path) else {
                return;
            };
            (file.modules.clone(), file.includes.clone())
        };
        for declaration in modules {
            self.walk_module(path, submodule_base, context, &declaration);
        }
        for include in includes {
            self.walk_include(path, context, &include);
        }
    }
}

const fn target_reachability(kind: CargoTargetKind) -> Reachability {
    let kind = match kind {
        CargoTargetKind::Library | CargoTargetKind::Binary => ReachabilityKind::Production,
        CargoTargetKind::Test => ReachabilityKind::Test,
        CargoTargetKind::Benchmark => ReachabilityKind::Benchmark,
        CargoTargetKind::Example => ReachabilityKind::Example,
        CargoTargetKind::BuildScript => ReachabilityKind::Build,
    };
    Reachability::from_kind(kind)
}
