//! Cargo roots and Rust source edges must form one closed, analyzable graph.

mod boundary;
mod include;

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use zrail_core::FindingSink;

use crate::source::{
    ModuleDeclaration, ModuleTarget, RustFileFacts, SourceSyntax, join_relative, module_target,
};

use super::RuleContext;

pub(super) fn evaluate(context: &RuleContext<'_>, findings: &mut FindingSink) {
    Walker::new(context, findings).run();
}

struct Walker<'a, 'f> {
    context: &'a RuleContext<'a>,
    findings: &'f mut FindingSink,
    facts: BTreeMap<&'a str, &'a RustFileFacts>,
    entries: BTreeMap<&'a str, crate::inventory::RepositoryEntryKind>,
    reached: BTreeSet<String>,
    seen_item_macros: BTreeSet<(String, String)>,
    seen_out_dir: BTreeSet<(String, String)>,
    visited: BTreeSet<(String, bool)>,
    queue: VecDeque<(String, bool)>,
}

impl<'a, 'f> Walker<'a, 'f> {
    fn new(context: &'a RuleContext<'a>, findings: &'f mut FindingSink) -> Self {
        Self {
            context,
            findings,
            facts: context
                .source
                .files
                .iter()
                .map(|file| (file.relative.as_str(), file))
                .collect(),
            entries: context
                .inventory
                .entries
                .iter()
                .map(|entry| (entry.relative.as_str(), entry.kind))
                .collect(),
            reached: BTreeSet::new(),
            seen_item_macros: BTreeSet::new(),
            seen_out_dir: BTreeSet::new(),
            visited: BTreeSet::new(),
            queue: VecDeque::new(),
        }
    }

    fn run(mut self) {
        self.seed_cargo_targets();
        while let Some((path, directory_owned)) = self.queue.pop_front() {
            self.walk_file(&path, directory_owned);
        }
        self.reject_orphans();
        self.reject_stale_item_macros();
        self.reject_stale_out_dir();
    }

    fn seed_cargo_targets(&mut self) {
        for package in &self.context.cargo.packages {
            if package.targets.is_empty() {
                let message = format!("Cargo package {:?} has no Rust target", package.name);
                self.missing(&package.manifest_path(), None, message);
            }
            for target in &package.targets {
                match join_relative(&package.directory, target) {
                    Ok(path) => self.follow(
                        &package.manifest_path(),
                        None,
                        path,
                        &format!("Cargo target {target:?}"),
                        true,
                        SourceSyntax::Items,
                    ),
                    Err(error) => self.resolution_error(
                        &package.manifest_path(),
                        None,
                        &error,
                        &format!("Cargo target {target:?}"),
                    ),
                }
            }
        }
    }

    fn walk_file(&mut self, path: &str, directory_owned: bool) {
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
            self.walk_module(path, directory_owned, &declaration);
        }
        for include in includes {
            self.walk_include(path, &include);
        }
    }

    fn item_macro_allowed(&self, path: &str, name: &str) -> bool {
        self.context
            .contract
            .source
            .rust
            .item_macros
            .iter()
            .any(|item_macro| item_macro.path == path && item_macro.name == name)
    }

    fn reject_stale_item_macros(&mut self) {
        for item_macro in &self.context.contract.source.rust.item_macros {
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
        declaration: &ModuleDeclaration,
    ) {
        let label = format!("module {:?}", declaration.name);
        match module_target(source, directory_owned, declaration) {
            Ok(ModuleTarget::Exact(path)) => {
                self.follow(
                    source,
                    declaration.span,
                    path,
                    &label,
                    false,
                    SourceSyntax::Items,
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
