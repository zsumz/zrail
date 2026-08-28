//! Bounded, origin-preserving resolution for associated item declarations.

use std::collections::BTreeMap;

use zrail_core::{AnalysisQuality, SourceSpan};

use super::super::{resolution, resolution::Route};
use crate::source::{
    AssociatedItemFact, CompilationDomain, FactNamespace, ObservedFact,
    include_bindings::{IncludeBindings, ResolvedOrigin, ResolvedTerminal},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::ResolutionUsage,
    operation_model::OperationSubjectOrigin,
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ResolveKey {
    file: String,
    path: String,
    quality: AnalysisQuality,
    guard: crate::source::SyntaxGuard,
    lexical_scope: Vec<SourceSpan>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct TraitRoute {
    pub(super) name: String,
    pub(super) origin: ResolvedOrigin,
}

pub(super) struct Resolver<'a> {
    bindings: &'a IncludeBindings,
    budget: &'a mut ProjectionBudget,
    cache: BTreeMap<ResolveKey, resolution::Resolution>,
}

impl<'a> Resolver<'a> {
    pub(super) fn new(bindings: &'a IncludeBindings, budget: &'a mut ProjectionBudget) -> Self {
        Self {
            bindings,
            budget,
            cache: BTreeMap::new(),
        }
    }

    pub(super) fn self_routes(
        &mut self,
        fact: &AssociatedItemFact,
        path: &str,
        file: &str,
    ) -> Result<Vec<Route>, ProjectionLimit> {
        Ok(self
            .resolve(fact, path, file)?
            .routes
            .into_iter()
            .filter(|route| {
                route.quality == AnalysisQuality::Exact
                    && match route.origin {
                        ResolvedOrigin::CrateLocal => matches!(
                            route.terminal,
                            ResolvedTerminal::Type | ResolvedTerminal::Constructor(_)
                        ),
                        ResolvedOrigin::External => route.terminal == ResolvedTerminal::Unknown,
                        ResolvedOrigin::Unknown => false,
                    }
            })
            .collect())
    }

    pub(super) fn trait_routes(
        &mut self,
        fact: &AssociatedItemFact,
        path: Option<&str>,
        file: &str,
    ) -> Result<BTreeMap<CompilationDomain, Vec<TraitRoute>>, ProjectionLimit> {
        let Some(path) = path else {
            return Ok(BTreeMap::new());
        };
        let mut routes = BTreeMap::<CompilationDomain, Vec<TraitRoute>>::new();
        for route in self.resolve(fact, path, file)?.routes {
            if route.quality == AnalysisQuality::Exact && route.origin != ResolvedOrigin::Unknown {
                routes.entry(route.domain).or_default().push(TraitRoute {
                    name: route.name,
                    origin: route.origin,
                });
            }
        }
        for values in routes.values_mut() {
            values.sort();
            values.dedup();
        }
        Ok(routes)
    }

    pub(super) fn local_trait_routes(
        &mut self,
        fact: &AssociatedItemFact,
        path: &str,
        file: &str,
    ) -> Result<Vec<(CompilationDomain, TraitRoute)>, ProjectionLimit> {
        Ok(self
            .trait_routes(fact, Some(path), file)?
            .into_iter()
            .filter_map(|(domain, routes)| {
                let [route] = routes.as_slice() else {
                    return None;
                };
                (route.origin == ResolvedOrigin::CrateLocal).then(|| (domain, route.clone()))
            })
            .collect())
    }

    fn resolve(
        &mut self,
        fact: &AssociatedItemFact,
        path: &str,
        file: &str,
    ) -> Result<resolution::Resolution, ProjectionLimit> {
        let key = ResolveKey {
            file: file.into(),
            path: path.into(),
            quality: fact.quality,
            guard: fact.guard.clone(),
            lexical_scope: fact.lexical_scope.clone(),
        };
        if let Some(resolution) = self.cache.get(&key) {
            return Ok(resolution.clone());
        }
        let observed = ObservedFact {
            name: path.into(),
            written: Some(path.into()),
            implicit_prelude: crate::source::ImplicitPreludeEligibility::Eligible,
            canonical: Vec::new(),
            span: Some(fact.span),
            quality: fact.quality,
            guard: fact.guard.clone(),
            lexical_scope: fact.lexical_scope.clone(),
            namespace: FactNamespace::Type,
            generic_shadow: None,
            associated_candidates: Vec::new(),
            inherits_parent_context: true,
        };
        let resolved = resolution::resolve(
            resolution::Request {
                bindings: self.bindings,
                file,
                fact: &observed,
                file_local: false,
                subject_origin: OperationSubjectOrigin::WrittenPath,
                written: path,
                usage: ResolutionUsage::Type,
                construction: None,
                root_lookup: None,
                generic_shadow: None,
            },
            self.budget,
        )?;
        self.cache.insert(key, resolved.clone());
        Ok(resolved)
    }
}
