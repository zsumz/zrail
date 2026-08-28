//! Source graph targets must remain regular, indexed Rust files inside declared roots.

use zrail_core::{Finding, SourceSpan, glob_matches};

use crate::{
    inventory::{FileClass, RepositoryEntryKind},
    source::{SourceSyntax, SubmoduleBase},
};

use super::{TraversalContext, Walker};

impl Walker<'_> {
    pub(super) fn follow_resolved(
        &mut self,
        origin: &str,
        span: Option<SourceSpan>,
        target: String,
        label: &str,
        submodule_base: SubmoduleBase,
        expected_syntax: SourceSyntax,
        context: TraversalContext,
    ) -> bool {
        if !self.under_roots(&target) || self.excluded(&target) {
            self.boundary(
                origin,
                span,
                format!("{label} resolves outside the indexed Rust roots: {target}"),
            );
            return false;
        }
        let indexable = target.to_ascii_lowercase().ends_with(".rs")
            || expected_syntax != SourceSyntax::Items
            || self
                .facts
                .get(target.as_str())
                .is_some_and(|file| file.class == FileClass::Generated);
        if !indexable {
            self.unresolved(
                origin,
                span,
                format!("{label} does not resolve to an indexable .rs file: {target}"),
            );
            return false;
        }
        match self.entries.get(target.as_str()) {
            None => {
                self.missing(
                    origin,
                    span,
                    format!("{label} source does not exist: {target}"),
                );
                false
            }
            Some(RepositoryEntryKind::File) => {
                self.reached
                    .entry(target.clone())
                    .and_modify(|current| *current = current.join(context.reachability))
                    .or_insert(context.reachability);
                self.reached_packages
                    .entry(target.clone())
                    .or_default()
                    .insert(context.package.clone());
                self.reached_domains
                    .entry(target.clone())
                    .or_default()
                    .insert(context.domain.clone());
                let Some(file) = self.facts.get(target.as_str()) else {
                    self.unresolved(
                        origin,
                        span,
                        format!("{label} source could not be indexed exactly: {target}"),
                    );
                    return false;
                };
                if file.syntax != expected_syntax {
                    self.unresolved(
                        origin,
                        span,
                        format!(
                            "{label} requires {} source but {target} parses as {}",
                            syntax_name(expected_syntax),
                            syntax_name(file.syntax)
                        ),
                    );
                    return false;
                }
                let state = (target, submodule_base, context);
                if self.visited.insert(state.clone()) {
                    self.queue.push_back(state);
                }
                true
            }
            Some(_) => {
                self.boundary(
                    origin,
                    span,
                    format!("{label} resolves to a symlink or non-file boundary: {target}"),
                );
                false
            }
        }
    }

    pub(super) fn reject_orphans(&mut self) {
        let orphans = self
            .inventory
            .rust_files
            .iter()
            .filter(|file| {
                !self.reached.contains_key(&file.relative)
                    && !self.generated_auxiliary(&file.relative)
            })
            .map(|file| file.relative.clone())
            .collect::<Vec<_>>();
        for path in orphans {
            self.findings.push(
                Finding::error(
                    "RUST-GRAPH-004",
                    "rust.source-graph.reachability",
                    "source-graph",
                    "Rust source is unreachable from every Cargo target",
                )
                .at(path, None)
                .with_help(
                    "declare it through Cargo or a reachable mod/include! edge, or remove it",
                ),
            );
        }
    }

    fn generated_auxiliary(&self, path: &str) -> bool {
        self.contract.source.rust.generated.iter().any(|generated| {
            generated.auxiliary.iter().any(|auxiliary| {
                crate::source::join_relative(&generated.root, auxiliary)
                    .is_ok_and(|candidate| candidate == path)
            })
        })
    }

    fn under_roots(&self, path: &str) -> bool {
        self.contract
            .repository
            .roots
            .iter()
            .any(|root| root == "." || path == root || path.starts_with(&format!("{root}/")))
    }

    fn excluded(&self, path: &str) -> bool {
        self.contract
            .repository
            .exclude
            .iter()
            .any(|pattern| glob_matches(pattern, path) || path.starts_with(&format!("{pattern}/")))
    }

    pub(super) fn boundary(&mut self, origin: &str, span: Option<SourceSpan>, message: String) {
        if !self.reported.insert((origin.into(), message.clone())) {
            return;
        }
        self.findings.push(
            Finding::error(
                "RUST-GRAPH-002",
                "rust.source-graph.boundary",
                "source-graph",
                message,
            )
            .at(origin, span),
        );
    }
}

fn syntax_name(syntax: SourceSyntax) -> &'static str {
    match syntax {
        SourceSyntax::Items => "Rust items",
        SourceSyntax::Expression => "a Rust expression",
        SourceSyntax::ImplItems => "Rust impl items",
        SourceSyntax::TraitItems => "Rust trait items",
    }
}
