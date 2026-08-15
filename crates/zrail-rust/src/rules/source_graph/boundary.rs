//! Source graph targets must remain regular, indexed Rust files inside declared roots.

use zrail_core::{AnalysisQuality, Finding, SourceSpan, path::glob_matches};

use crate::{
    inventory::{FileClass, RepositoryEntryKind},
    source::{ResolutionError, SourceSyntax},
};

use super::Walker;

impl Walker<'_, '_> {
    pub(super) fn follow(
        &mut self,
        origin: &str,
        span: Option<SourceSpan>,
        target: String,
        label: &str,
        directory_owned: bool,
        expected_syntax: SourceSyntax,
    ) {
        if !self.under_roots(&target) || self.excluded(&target) {
            self.boundary(
                origin,
                span,
                format!("{label} resolves outside the indexed Rust roots: {target}"),
            );
            return;
        }
        let indexable = target.to_ascii_lowercase().ends_with(".rs")
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
            return;
        }
        match self.entries.get(target.as_str()) {
            None => self.missing(
                origin,
                span,
                format!("{label} source does not exist: {target}"),
            ),
            Some(RepositoryEntryKind::File) => {
                self.reached.insert(target.clone());
                let Some(file) = self.facts.get(target.as_str()) else {
                    self.unresolved(
                        origin,
                        span,
                        format!("{label} source could not be indexed exactly: {target}"),
                    );
                    return;
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
                    return;
                }
                let state = (target, directory_owned);
                if !self.visited.insert(state.clone()) {
                    return;
                }
                self.queue.push_back(state);
            }
            Some(_) => self.boundary(
                origin,
                span,
                format!("{label} resolves to a symlink or non-file boundary: {target}"),
            ),
        }
    }

    pub(super) fn reject_orphans(&mut self) {
        let orphans = self
            .context
            .inventory
            .rust_files
            .iter()
            .filter(|file| {
                !self.reached.contains(&file.relative) && !self.generated_auxiliary(&file.relative)
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
        self.context
            .contract
            .source
            .rust
            .generated
            .iter()
            .any(|generated| {
                generated.auxiliary.iter().any(|auxiliary| {
                    crate::source::join_relative(&generated.root, auxiliary)
                        .is_ok_and(|candidate| candidate == path)
                })
            })
    }

    pub(super) fn resolution_error(
        &mut self,
        origin: &str,
        span: Option<SourceSpan>,
        error: &ResolutionError,
        label: &str,
    ) {
        let message = format!("{label} cannot be resolved: {}", error.message());
        match error {
            ResolutionError::Escape(_) => self.boundary(origin, span, message),
            ResolutionError::Unresolved(_) => self.unresolved(origin, span, message),
        }
    }

    pub(super) fn missing(&mut self, origin: &str, span: Option<SourceSpan>, message: String) {
        self.findings.push(
            Finding::error(
                "RUST-GRAPH-001",
                "rust.source-graph.presence",
                "source-graph",
                message,
            )
            .at(origin, span),
        );
    }

    fn under_roots(&self, path: &str) -> bool {
        self.context
            .contract
            .repository
            .roots
            .iter()
            .any(|root| root == "." || path == root || path.starts_with(&format!("{root}/")))
    }

    fn excluded(&self, path: &str) -> bool {
        self.context
            .contract
            .repository
            .exclude
            .iter()
            .any(|pattern| glob_matches(pattern, path) || path.starts_with(&format!("{pattern}/")))
    }

    fn boundary(&mut self, origin: &str, span: Option<SourceSpan>, message: String) {
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

    pub(super) fn unresolved(&mut self, origin: &str, span: Option<SourceSpan>, message: String) {
        self.findings.push(
            Finding::error(
                "RUST-GRAPH-003",
                "rust.source-graph.analysis",
                "source-graph",
                message,
            )
            .at(origin, span)
            .with_analysis(AnalysisQuality::Unresolved)
            .with_help("replace the boundary with a literal repository-local .rs source path"),
        );
    }
}

fn syntax_name(syntax: SourceSyntax) -> &'static str {
    match syntax {
        SourceSyntax::Items => "Rust items",
        SourceSyntax::Expression => "a Rust expression",
    }
}
