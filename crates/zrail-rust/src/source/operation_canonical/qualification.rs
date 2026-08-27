//! Explicit trait qualifications are proven in the occurrence's own scope.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::resolution;
use crate::source::{
    CompilationDomain, SyntaxGuard,
    include_bindings::{IncludeBindings, ResolvedOrigin, ResolvedTerminal},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::ResolutionUsage,
    operation_model::{OperationSubjectOrigin, QualifiedOperationSubject},
};

pub(super) fn classify(
    subject: Option<&QualifiedOperationSubject>,
    result: &mut resolution::Resolution,
    catalog: &super::associated::Catalog,
    context: &SyntaxGuard,
    written: &str,
    bindings: &IncludeBindings,
    file: &str,
    budget: &mut ProjectionBudget,
) -> Result<Disposition, ProjectionLimit> {
    let occurrence = resolve(subject, bindings, file, budget)?;
    if subject.is_some_and(|subject| subject.direct_trait_item) {
        for route in &mut result.routes {
            let selection = occurrence.selection(&route.domain);
            catalog.classify_value(route, context, selection);
        }
        return Ok(Disposition::AssociatedItem(occurrence.quality));
    }
    if subject.is_some_and(|subject| subject.force_unresolved) {
        result.unresolved = true;
        for route in &mut result.routes {
            route.name = written.into();
            route.quality = AnalysisQuality::Unresolved;
            route.origin = ResolvedOrigin::Unknown;
            route.terminal = ResolvedTerminal::Unknown;
        }
        return Ok(Disposition::ConstructionCandidate);
    }
    for route in &mut result.routes {
        let selection = occurrence.selection(&route.domain);
        catalog.classify_value(route, context, selection);
    }
    Ok(Disposition::ConstructionCandidate)
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Disposition {
    ConstructionCandidate,
    AssociatedItem(AnalysisQuality),
}

pub(super) struct OccurrenceTraits {
    explicit: bool,
    exact: BTreeMap<CompilationDomain, String>,
    quality: AnalysisQuality,
}

impl Default for OccurrenceTraits {
    fn default() -> Self {
        Self {
            explicit: false,
            exact: BTreeMap::new(),
            quality: AnalysisQuality::Exact,
        }
    }
}

impl OccurrenceTraits {
    pub(super) fn selection(&self, domain: &CompilationDomain) -> TraitSelection<'_> {
        if self.explicit {
            TraitSelection::Explicit(self.exact.get(domain).map(String::as_str))
        } else {
            TraitSelection::Ordinary
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum TraitSelection<'a> {
    Ordinary,
    Explicit(Option<&'a str>),
}

pub(super) fn resolve(
    subject: Option<&QualifiedOperationSubject>,
    bindings: &IncludeBindings,
    file: &str,
    budget: &mut ProjectionBudget,
) -> Result<OccurrenceTraits, ProjectionLimit> {
    let Some(subject) = subject else {
        return Ok(OccurrenceTraits::default());
    };
    if !subject.explicit_trait {
        return Ok(OccurrenceTraits::default());
    }
    let mut occurrence = OccurrenceTraits {
        explicit: true,
        exact: BTreeMap::new(),
        quality: AnalysisQuality::Unresolved,
    };
    let Some(fact) = &subject.trait_identity else {
        return Ok(occurrence);
    };
    let written = fact.written.as_deref().unwrap_or(&fact.name);
    let resolved = resolution::resolve(
        resolution::Request {
            bindings,
            file,
            fact,
            file_local: false,
            subject_origin: OperationSubjectOrigin::WrittenPath,
            written,
            usage: ResolutionUsage::Type,
            construction: None,
        },
        budget,
    )?;
    let complete = !resolved.unresolved
        && resolved.expected > 0
        && resolved.routes.len() == resolved.expected
        && resolved
            .routes
            .iter()
            .all(|route| exact_trait_route(route, bindings, file, &fact.guard));
    let domains = resolved
        .routes
        .iter()
        .map(|route| route.domain.clone())
        .collect::<BTreeSet<_>>();
    let mut by_domain = BTreeMap::<CompilationDomain, BTreeSet<_>>::new();
    for route in resolved.routes {
        by_domain.entry(route.domain).or_default().insert((
            route.name,
            route.quality,
            route.origin,
            route.terminal,
        ));
    }
    for (domain, routes) in by_domain {
        let routes = routes.into_iter().collect::<Vec<_>>();
        let [(name, AnalysisQuality::Exact, origin, terminal)] = routes.as_slice() else {
            continue;
        };
        if !exact_trait_identity(
            name,
            *origin,
            *terminal,
            &domain,
            bindings,
            file,
            &fact.guard,
        ) {
            continue;
        }
        occurrence.exact.insert(domain, name.clone());
    }
    if complete
        && domains
            .iter()
            .all(|domain| occurrence.exact.contains_key(domain))
    {
        occurrence.quality = AnalysisQuality::Exact;
    }
    Ok(occurrence)
}

fn exact_trait_route(
    route: &resolution::Route,
    bindings: &IncludeBindings,
    file: &str,
    guard: &SyntaxGuard,
) -> bool {
    route.quality == AnalysisQuality::Exact
        && exact_trait_identity(
            &route.name,
            route.origin,
            route.terminal,
            &route.domain,
            bindings,
            file,
            guard,
        )
}

fn exact_trait_identity(
    name: &str,
    origin: ResolvedOrigin,
    terminal: ResolvedTerminal,
    domain: &CompilationDomain,
    bindings: &IncludeBindings,
    file: &str,
    guard: &SyntaxGuard,
) -> bool {
    match (origin, terminal) {
        (ResolvedOrigin::CrateLocal, ResolvedTerminal::Type) => true,
        (ResolvedOrigin::External, ResolvedTerminal::Type | ResolvedTerminal::Unknown) => {
            let root = name.trim_start_matches("::").split("::").next();
            root.is_some_and(|root| {
                matches!(root, "std" | "core")
                    || bindings
                        .active_instances(file, guard)
                        .iter()
                        .any(|instance| {
                            bindings
                                .instances
                                .get(*instance)
                                .is_some_and(|source| &source.domain == domain)
                                && bindings.is_extern_root(*instance, root)
                        })
            })
        }
        _ => false,
    }
}
