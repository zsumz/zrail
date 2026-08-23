//! Ordinary paths retain every namespace identity introduced by include splices.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{AnalysisQuality, Finding};

use super::{
    CompilationIncludeEdge, CompilationModuleEdge, CompilationRoot, ImportBindingFact,
    ObservedFact, RustFileFacts, SourceIndex, SourceInstanceId, SourceInstances, SourceSyntax,
    SyntaxGuard,
};

const MAX_PROJECTED_IDENTITIES: usize = 64;
const MAX_PROJECTED_FACTS_PER_FILE: usize = 50_000;

pub(super) struct IncludeBindings {
    pub(super) files: BTreeMap<String, Vec<ImportBindingFact>>,
    pub(super) instances: SourceInstances,
}

#[derive(Clone)]
pub(super) struct BindingSite {
    pub(super) binding: ImportBindingFact,
    pub(super) instance: SourceInstanceId,
    pub(super) crossed_include: bool,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct ResolvedPath {
    pub(super) name: String,
    pub(super) quality: AnalysisQuality,
    pub(super) crossed_include: bool,
}

struct CandidateAggregate {
    instances: usize,
    quality: AnalysisQuality,
    production: bool,
    crossed_include: bool,
}

impl Default for CandidateAggregate {
    fn default() -> Self {
        Self {
            instances: 0,
            quality: AnalysisQuality::Exact,
            production: false,
            crossed_include: false,
        }
    }
}

impl Default for ResolvedPath {
    fn default() -> Self {
        Self {
            name: String::new(),
            quality: AnalysisQuality::Exact,
            crossed_include: false,
        }
    }
}

impl IncludeBindings {
    pub(super) fn collect(
        index: &SourceIndex,
        roots: &[CompilationRoot],
        modules: &[CompilationModuleEdge],
        includes: &[CompilationIncludeEdge],
    ) -> Self {
        Self {
            files: index
                .files
                .iter()
                .map(|file| (file.relative.clone(), file.import_bindings.clone()))
                .collect(),
            instances: SourceInstances::build(roots, modules, includes),
        }
    }

    pub(super) fn apply(&self, file: &mut RustFileFacts) -> Option<Finding> {
        if !self.instances.complete {
            return has_written_facts(file).then(|| unresolved(file, None));
        }
        let mut uncertain = None;
        let project_local = file.syntax == SourceSyntax::Expression;
        project(
            self,
            &file.relative,
            &mut file.paths,
            project_local,
            &mut uncertain,
        );
        project(
            self,
            &file.relative,
            &mut file.calls,
            project_local,
            &mut uncertain,
        );
        uncertain.map(|span| unresolved(file, Some(span)))
    }

    pub(super) fn active_instances(&self, file: &str, guard: SyntaxGuard) -> Vec<SourceInstanceId> {
        self.instances
            .for_file(file)
            .iter()
            .copied()
            .filter(|id| {
                self.instances.get(*id).is_some_and(|instance| {
                    guard.available_in(SyntaxGuard::for_test_only(
                        instance.domain.mode.enables_cfg_test(),
                    ))
                })
            })
            .collect()
    }
}

fn project(
    bindings: &IncludeBindings,
    file: &str,
    facts: &mut Vec<ObservedFact>,
    project_local: bool,
    uncertain: &mut Option<zrail_core::SourceSpan>,
) {
    let mut additions = Vec::new();
    for fact in facts.iter().filter(|fact| fact.written.is_some()) {
        let instances = bindings.active_instances(file, fact.guard);
        if instances.is_empty() {
            continue;
        }
        let mut aggregate = BTreeMap::<String, CandidateAggregate>::new();
        let mut compatible = true;
        let mut common = None;
        for instance in &instances {
            let mut seen = BTreeSet::new();
            let resolved = bindings.resolve_written(
                *instance,
                fact.written.as_deref().unwrap_or(&fact.name),
                &fact.lexical_scope,
                &mut seen,
                0,
            );
            compatible &= resolved.len() == 1;
            let names = resolved
                .iter()
                .map(|candidate| candidate.name.as_str())
                .collect::<BTreeSet<_>>();
            let only = (names.len() == 1)
                .then(|| names.iter().next().copied())
                .flatten();
            common = match (common, only) {
                (None, name) => name.map(str::to_owned),
                (Some(current), Some(name)) if current == name => Some(current),
                _ => {
                    compatible = false;
                    None
                }
            };
            for candidate in resolved {
                let entry = aggregate.entry(candidate.name).or_default();
                entry.instances += 1;
                entry.quality = entry.quality.max(candidate.quality);
                entry.crossed_include |= candidate.crossed_include;
                entry.production |= bindings
                    .instances
                    .get(*instance)
                    .is_some_and(|source| !source.domain.mode.enables_cfg_test());
            }
        }
        if aggregate.len() > MAX_PROJECTED_IDENTITIES {
            *uncertain = uncertain.or(fact.span);
            continue;
        }
        for (name, candidate) in aggregate {
            if !candidate.crossed_include && !project_local {
                continue;
            }
            let complete = compatible && candidate.instances == instances.len();
            let quality = if candidate.quality == AnalysisQuality::Unresolved {
                if candidate.crossed_include {
                    *uncertain = uncertain.or(fact.span);
                }
                AnalysisQuality::Unresolved
            } else if complete {
                AnalysisQuality::Exact
            } else {
                AnalysisQuality::Conservative
            };
            if name == fact.name {
                continue;
            }
            additions.push(ObservedFact {
                name,
                written: None,
                canonical: Vec::new(),
                span: fact.span,
                quality,
                guard: if fact.guard == SyntaxGuard::TestOnly || !candidate.production {
                    SyntaxGuard::TestOnly
                } else {
                    SyntaxGuard::Ordinary
                },
                lexical_scope: fact.lexical_scope.clone(),
            });
            if additions.len() > MAX_PROJECTED_FACTS_PER_FILE {
                *uncertain = uncertain.or(fact.span);
                return;
            }
        }
    }
    additions.sort_by(|left, right| {
        (&left.name, left.span, left.guard).cmp(&(&right.name, right.span, right.guard))
    });
    additions.dedup_by(|left, right| {
        left.name == right.name && left.span == right.span && left.guard == right.guard
    });
    facts.extend(additions);
}

fn has_written_facts(file: &RustFileFacts) -> bool {
    file.paths
        .iter()
        .chain(&file.calls)
        .any(|fact| fact.written.is_some())
}

fn unresolved(file: &RustFileFacts, span: Option<zrail_core::SourceSpan>) -> Finding {
    Finding::error(
        "RUST-INCLUDE-002",
        "rust.source.include-bindings",
        "source",
        "include-spliced ordinary path bindings could not be resolved completely",
    )
    .at(&file.relative, span)
    .with_analysis(AnalysisQuality::Unresolved)
    .with_help("reduce include or import indirection before trusting path and call authority")
}
