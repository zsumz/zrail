//! Cargo roots and Rust source edges must form one closed, analyzable graph.

mod boundary;
mod include;

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use zrail_core::{Contract, Finding, FindingSink};

use crate::{
    cargo::{CargoTargetKind, CargoWorkspace},
    inventory::{RepositoryEntryKind, RepositoryInventory},
    source::{
        ModuleDeclaration, ModuleTarget, Reachability, RustFileFacts, SourceIndex, SourceSyntax,
        join_relative, module_target,
    },
};

pub(crate) fn analyze(
    contract: &Contract,
    inventory: &RepositoryInventory,
    cargo: &CargoWorkspace,
    source: &SourceIndex,
) -> (BTreeMap<String, Reachability>, Vec<Finding>) {
    Walker::new(contract, inventory, cargo, source).run()
}

struct Walker<'a> {
    contract: &'a Contract,
    inventory: &'a RepositoryInventory,
    cargo: &'a CargoWorkspace,
    findings: FindingSink,
    facts: BTreeMap<&'a str, &'a RustFileFacts>,
    entries: BTreeMap<&'a str, RepositoryEntryKind>,
    reached: BTreeMap<String, Reachability>,
    seen_item_macros: BTreeSet<(String, String)>,
    seen_out_dir: BTreeSet<(String, String)>,
    reported: BTreeSet<(String, String)>,
    visited: BTreeSet<(String, bool, Reachability)>,
    queue: VecDeque<(String, bool, Reachability)>,
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
            seen_item_macros: BTreeSet::new(),
            seen_out_dir: BTreeSet::new(),
            reported: BTreeSet::new(),
            visited: BTreeSet::new(),
            queue: VecDeque::new(),
        }
    }

    fn run(mut self) -> (BTreeMap<String, Reachability>, Vec<Finding>) {
        self.seed_cargo_targets();
        while let Some((path, directory_owned, reachability)) = self.queue.pop_front() {
            self.walk_file(&path, directory_owned, reachability);
        }
        self.reject_orphans();
        self.reject_stale_item_macros();
        self.reject_stale_out_dir();
        (self.reached, self.findings.into_findings())
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
                        reachability,
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

    fn walk_file(&mut self, path: &str, directory_owned: bool, reachability: Reachability) {
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
            self.walk_module(path, directory_owned, reachability, &declaration);
        }
        for include in includes {
            self.walk_include(path, reachability, &include);
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

    fn walk_module(
        &mut self,
        source: &str,
        directory_owned: bool,
        reachability: Reachability,
        declaration: &ModuleDeclaration,
    ) {
        let label = format!("module {:?}", declaration.name);
        let target_reachability = if declaration.cfg_test {
            Reachability::TestOnly
        } else {
            reachability
        };
        match module_target(source, directory_owned, declaration) {
            Ok(ModuleTarget::Exact(path)) => {
                self.follow(
                    source,
                    declaration.span,
                    path,
                    &label,
                    false,
                    SourceSyntax::Items,
                    target_reachability,
                );
            }
            Ok(ModuleTarget::Search { direct, nested }) => {
                let candidates = [direct, nested]
                    .into_iter()
                    .filter(|path| self.entries.contains_key(path.as_str()))
                    .collect::<Vec<_>>();
                match candidates.as_slice() {
                    [path] => {
                        self.follow(
                            source,
                            declaration.span,
                            path.clone(),
                            &label,
                            false,
                            SourceSyntax::Items,
                            target_reachability,
                        );
                    }
                    [] => self.missing(
                        source,
                        declaration.span,
                        format!("{label} has no source file at either Rust module path"),
                    ),
                    _ => self.missing(
                        source,
                        declaration.span,
                        format!("{label} is ambiguous because both Rust module paths exist"),
                    ),
                }
            }
            Err(error) => self.resolution_error(source, declaration.span, &error, &label),
        }
    }
}
