//! Stateful exact traversal across Cargo roots, modules, and include edges.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use zrail_core::{Contract, Finding};

use crate::{
    cargo::{CargoWorkspace, ResolvedFeatureWorld},
    inventory::{RepositoryEntryKind, RepositoryInventory},
    source::{
        CompilationDomain, CompilationIncludeEdge, CompilationModuleEdge, CompilationRoot,
        Reachability, ResolvedModuleEdge, RustFileFacts, SourceIndex, SourceSyntax, SubmoduleBase,
        join_relative,
    },
};

use super::{SourceGraphAnalysis, compilation::target_domains};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct TraversalContext {
    pub(super) reachability: Reachability,
    pub(super) package: String,
    pub(super) domain: CompilationDomain,
    pub(super) guard: crate::source::SyntaxGuard,
}

impl TraversalContext {
    pub(super) fn with_guard(&self, guard: &crate::source::SyntaxGuard) -> Option<Self> {
        let guard = self.guard.combine(guard);
        if !guard.availability_in_domain(&self.domain).is_available() {
            return None;
        }
        Some(Self {
            reachability: if guard.is_test_only() {
                Reachability::test()
            } else {
                self.reachability
            },
            package: self.package.clone(),
            domain: self.domain.clone(),
            guard,
        })
    }
}

pub(super) struct Walker<'a> {
    pub(super) contract: &'a Contract,
    pub(super) inventory: &'a RepositoryInventory,
    pub(super) cargo: &'a CargoWorkspace,
    pub(super) feature_worlds: &'a [ResolvedFeatureWorld],
    pub(super) findings: Vec<Finding>,
    pub(super) facts: BTreeMap<(&'a str, SourceSyntax), &'a RustFileFacts>,
    pub(super) entries: BTreeMap<&'a str, RepositoryEntryKind>,
    pub(super) reached: BTreeMap<String, Reachability>,
    pub(super) reached_packages: BTreeMap<String, BTreeSet<String>>,
    pub(super) reached_domains: BTreeMap<String, BTreeSet<CompilationDomain>>,
    pub(super) seen_out_dir: BTreeSet<(String, String)>,
    pub(super) reported: BTreeSet<(String, String)>,
    pub(super) module_edges: BTreeSet<ResolvedModuleEdge>,
    pub(super) compilation_edges: BTreeSet<CompilationModuleEdge>,
    pub(super) compilation_includes: BTreeSet<CompilationIncludeEdge>,
    pub(super) compilation_roots: BTreeSet<CompilationRoot>,
    pub(super) visited: BTreeSet<(String, SourceSyntax, SubmoduleBase, TraversalContext)>,
    pub(super) queue: VecDeque<(String, SourceSyntax, SubmoduleBase, TraversalContext)>,
}

impl<'a> Walker<'a> {
    pub(super) fn new(
        contract: &'a Contract,
        inventory: &'a RepositoryInventory,
        cargo: &'a CargoWorkspace,
        feature_worlds: &'a [ResolvedFeatureWorld],
        source: &'a SourceIndex,
    ) -> Self {
        Self {
            contract,
            inventory,
            cargo,
            feature_worlds,
            findings: Vec::new(),
            facts: source
                .files
                .iter()
                .map(|file| ((file.relative.as_str(), file.syntax), file))
                .collect(),
            entries: inventory
                .entries
                .iter()
                .map(|entry| (entry.relative.as_str(), entry.kind))
                .collect(),
            reached: BTreeMap::new(),
            reached_packages: BTreeMap::new(),
            reached_domains: BTreeMap::new(),
            seen_out_dir: BTreeSet::new(),
            reported: BTreeSet::new(),
            module_edges: BTreeSet::new(),
            compilation_edges: BTreeSet::new(),
            compilation_includes: BTreeSet::new(),
            compilation_roots: BTreeSet::new(),
            visited: BTreeSet::new(),
            queue: VecDeque::new(),
        }
    }

    pub(super) fn run(mut self) -> SourceGraphAnalysis {
        self.seed_cargo_targets();
        while let Some((path, syntax, submodule_base, context)) = self.queue.pop_front() {
            self.walk_file(&path, syntax, submodule_base, &context);
        }
        self.reject_orphans();
        self.reject_stale_out_dir();
        SourceGraphAnalysis {
            reachability: self.reached,
            packages: self.reached_packages,
            compilation_domains: self.reached_domains,
            compilation_roots: self.compilation_roots.into_iter().collect(),
            compilation_edges: self.compilation_edges.into_iter().collect(),
            compilation_includes: self.compilation_includes.into_iter().collect(),
            module_edges: self.module_edges.into_iter().collect(),
            findings: self.findings,
        }
    }

    fn seed_cargo_targets(&mut self) {
        for package in &self.cargo.packages {
            if package.targets.is_empty() {
                let message = format!("Cargo package {:?} has no Rust target", package.name);
                self.missing(&package.manifest_path(), None, message);
            }
            let feature_worlds = if self.feature_worlds.is_empty() {
                vec![(None, BTreeSet::new())]
            } else {
                self.feature_worlds
                    .iter()
                    .map(|world| {
                        (
                            Some(world.name.clone()),
                            world.packages[&package.name]
                                .active
                                .iter()
                                .cloned()
                                .collect(),
                        )
                    })
                    .collect()
            };
            for target in &package.targets {
                for (feature_world, active_features) in &feature_worlds {
                    if !super::feature_worlds::target_enabled(
                        target,
                        feature_world.as_deref(),
                        active_features,
                    ) {
                        continue;
                    }
                    for (mode, reachability) in target_domains(target.kind) {
                        let domain = CompilationDomain {
                            package: package.name.clone(),
                            edition: package.edition.clone(),
                            target: target.name.clone(),
                            mode,
                            feature_world: feature_world.clone(),
                            active_features: active_features.clone(),
                        };
                        match join_relative(&package.directory, &target.path) {
                            Ok(path) => self.follow_root(
                                &package.manifest_path(),
                                None,
                                path,
                                &format!("Cargo target {:?}", target.path),
                                SubmoduleBase::SourceParent,
                                SourceSyntax::Items,
                                TraversalContext {
                                    reachability,
                                    package: package.name.clone(),
                                    domain,
                                    guard: crate::source::SyntaxGuard::Ordinary,
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
        }
    }

    fn walk_file(
        &mut self,
        path: &str,
        syntax: SourceSyntax,
        submodule_base: SubmoduleBase,
        context: &TraversalContext,
    ) {
        let (modules, includes) = {
            let Some(file) = self.facts.get(&(path, syntax)) else {
                return;
            };
            (file.modules.clone(), file.includes.clone())
        };
        for declaration in modules {
            self.walk_module(path, syntax, submodule_base, context, &declaration);
        }
        for include in includes {
            self.walk_include(path, syntax, context, &include);
        }
    }
}
