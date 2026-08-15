//! Cargo roots and Rust source edges must form one closed, analyzable graph.

mod boundary;
mod external_module;
mod include;

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use zrail_core::{Contract, Finding, FindingSink};

use crate::{
    cargo::{CargoTargetKind, CargoWorkspace},
    inventory::{RepositoryEntryKind, RepositoryInventory},
    source::{Reachability, RustFileFacts, SourceIndex, SourceSyntax, join_relative},
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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct TraversalContext {
    reachability: Reachability,
    package: String,
}

impl TraversalContext {
    fn with_test_guard(&self, guarded: bool) -> Self {
        Self {
            reachability: if guarded {
                Reachability::TestOnly
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
    findings: FindingSink,
    facts: BTreeMap<&'a str, &'a RustFileFacts>,
    entries: BTreeMap<&'a str, RepositoryEntryKind>,
    reached: BTreeMap<String, Reachability>,
    reached_packages: BTreeMap<String, BTreeSet<String>>,
    seen_item_macros: BTreeSet<(String, String)>,
    seen_out_dir: BTreeSet<(String, String)>,
    reported: BTreeSet<(String, String)>,
    visited: BTreeSet<(String, bool, TraversalContext)>,
    queue: VecDeque<(String, bool, TraversalContext)>,
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
            findings: FindingSink::default(),
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
            seen_item_macros: BTreeSet::new(),
            seen_out_dir: BTreeSet::new(),
            reported: BTreeSet::new(),
            visited: BTreeSet::new(),
            queue: VecDeque::new(),
        }
    }

    fn run(mut self) -> SourceGraphAnalysis {
        self.seed_cargo_targets();
        while let Some((path, directory_owned, context)) = self.queue.pop_front() {
            self.walk_file(&path, directory_owned, &context);
        }
        self.reject_orphans();
        self.reject_stale_item_macros();
        self.reject_stale_out_dir();
        SourceGraphAnalysis {
            reachability: self.reached,
            packages: self.reached_packages,
            findings: self.findings.into_findings(),
        }
    }

    fn seed_cargo_targets(&mut self) {
        for package in &self.cargo.packages {
            if package.targets.is_empty() {
                let message = format!("Cargo package {:?} has no Rust target", package.name);
                self.missing(&package.manifest_path(), None, message);
            }
            for target in &package.targets {
                let reachability = if target.kind == CargoTargetKind::Test {
                    Reachability::TestOnly
                } else {
                    Reachability::Production
                };
                match join_relative(&package.directory, &target.path) {
                    Ok(path) => self.follow(
                        &package.manifest_path(),
                        None,
                        path,
                        &format!("Cargo target {:?}", target.path),
                        true,
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

    fn walk_file(&mut self, path: &str, directory_owned: bool, context: &TraversalContext) {
        let Some(file) = self.facts.get(path) else {
            return;
        };
        let modules = file.modules.clone();
        let includes = file.includes.clone();
        let item_macros = file.item_macros.clone();
        for invocation in item_macros {
            if self.item_macro_allowed(path, &invocation.name) {
                self.seen_item_macros
                    .insert((path.to_owned(), invocation.name));
            } else {
                self.unresolved(
                    path,
                    invocation.span,
                    format!(
                        "item-position macro {}! may create source edges that static analysis cannot resolve",
                        invocation.name
                    ),
                );
            }
        }
        for declaration in modules {
            self.walk_module(path, directory_owned, context, &declaration);
        }
        for include in includes {
            self.walk_include(path, context, &include);
        }
    }

    fn item_macro_allowed(&self, path: &str, name: &str) -> bool {
        self.contract
            .source
            .rust
            .item_macros
            .iter()
            .any(|item_macro| item_macro.path == path && item_macro.name == name)
    }

    fn reject_stale_item_macros(&mut self) {
        for item_macro in &self.contract.source.rust.item_macros {
            if self
                .seen_item_macros
                .contains(&(item_macro.path.clone(), item_macro.name.clone()))
            {
                continue;
            }
            self.findings.push(
                zrail_core::Finding::error(
                    "RUST-GRAPH-005",
                    "rust.source-graph.item-macro",
                    "source-graph",
                    format!(
                        "item macro exemption {}! matches no reachable invocation",
                        item_macro.name
                    ),
                )
                .at(&item_macro.path, None)
                .because(&item_macro.reason),
            );
        }
    }
}
