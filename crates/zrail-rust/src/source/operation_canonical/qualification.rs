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
) -> Result<(), ProjectionLimit> {
    let occurrence = resolve(subject, bindings, file, budget)?;
    if subject.is_some_and(|subject| subject.force_unresolved) {
        result.unresolved = true;
        for route in &mut result.routes {
            route.name = written.into();
            route.quality = AnalysisQuality::Unresolved;
            route.origin = ResolvedOrigin::Unknown;
            route.terminal = ResolvedTerminal::Unknown;
        }
        return Ok(());
    }
    for route in &mut result.routes {
        let selection = occurrence.selection(&route.domain);
        catalog.classify_value(route, context, selection);
    }
    Ok(())
}

#[derive(Default)]
pub(super) struct OccurrenceTraits {
    explicit: bool,
    exact: BTreeMap<CompilationDomain, String>,
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
        let [(name, AnalysisQuality::Exact, ResolvedOrigin::CrateLocal, ResolvedTerminal::Type)] =
            routes.as_slice()
        else {
            continue;
        };
        occurrence.exact.insert(domain, name.clone());
    }
    Ok(occurrence)
}
