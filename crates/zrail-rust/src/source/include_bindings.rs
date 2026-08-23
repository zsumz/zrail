//! Ordinary paths retain every namespace identity introduced by include splices.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::{
    CompilationIncludeEdge, CompilationModuleEdge, CompilationRoot, ImportBindingFact,
    ObservedFact, SourceIndex, SourceInstanceId, SourceInstances, SyntaxGuard,
    include_binding_catalog::FileBindings,
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
};

const MAX_PROJECTED_IDENTITIES: usize = 64;

pub(super) struct IncludeBindings {
    pub(super) files: BTreeMap<String, FileBindings>,
    pub(super) inline_module_scopes: BTreeMap<String, BTreeSet<zrail_core::SourceSpan>>,
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
                .map(|file| {
                    (
                        file.relative.clone(),
                        FileBindings::collect(&file.import_bindings),
                    )
                })
                .collect(),
            inline_module_scopes: index
                .files
                .iter()
                .map(|file| {
                    (
                        file.relative.clone(),
                        file.inline_module_scopes.iter().copied().collect(),
                    )
                })
                .collect(),
            instances: SourceInstances::build(roots, modules, includes),
        }
    }

    pub(super) fn active_instances(
        &self,
        file: &str,
        guard: SyntaxGuard,
        budget: &mut ProjectionBudget,
    ) -> Result<Vec<SourceInstanceId>, ProjectionLimit> {
        let mut active = Vec::new();
        for id in self.instances.for_file(file) {
            budget.consume_work()?;
            if self.instances.get(*id).is_some_and(|instance| {
                guard.available_in(SyntaxGuard::for_test_only(
                    instance.domain.mode.enables_cfg_test(),
                ))
            }) {
                active.push(*id);
            }
        }
        Ok(active)
    }
}

pub(super) fn project(
    bindings: &IncludeBindings,
    file: &str,
    facts: &[ObservedFact],
    project_local: bool,
    uncertain: &mut Option<zrail_core::SourceSpan>,
    budget: &mut ProjectionBudget,
    remaining_file_facts: &mut usize,
) -> Result<Vec<ObservedFact>, ProjectionLimit> {
    let mut additions = Vec::new();
    for fact in facts.iter().filter(|fact| fact.written.is_some()) {
        budget.consume_work()?;
        let instances = bindings.active_instances(file, fact.guard, budget)?;
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
                budget,
            )?;
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
        }
    }
    additions.sort_by(|left, right| {
        (&left.name, left.span, left.guard).cmp(&(&right.name, right.span, right.guard))
    });
    additions.dedup_by(|left, right| {
        left.name == right.name && left.span == right.span && left.guard == right.guard
    });
    for _ in &additions {
        budget.retain_fact(remaining_file_facts)?;
    }
    Ok(additions)
}
