//! Associated values are cataloged by canonical self type, not impl location.

#[path = "associated/classification.rs"]
mod classification;
#[path = "associated/routes.rs"]
mod routes;

use std::collections::{BTreeMap, BTreeSet};

use super::resolution::Route;
use crate::source::{
    AssociatedItemFact, CompilationDomain, SourceIndex, SourceSyntax, SyntaxGuard,
    associated_items::AssociatedItemKind,
    include_bindings::{IncludeBindings, ResolvedOrigin},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
};

#[derive(Default)]
pub(super) struct Catalog {
    entries: BTreeMap<Key, BTreeMap<TraitIdentity, Vec<SyntaxGuard>>>,
    external_self: BTreeSet<Key>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Key {
    domain: CompilationDomain,
    self_type: String,
    item: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum TraitIdentity {
    Inherent,
    Canonical(String),
    Unresolved(String),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct DefaultKey {
    domain: CompilationDomain,
    trait_name: String,
}

struct DefaultImpl {
    domain: CompilationDomain,
    self_type: String,
    trait_name: String,
    guard: SyntaxGuard,
    external_self: bool,
}

impl Catalog {
    pub(super) fn collect(
        index: &SourceIndex,
        bindings: &IncludeBindings,
        budget: &mut ProjectionBudget,
    ) -> Result<Self, ProjectionLimit> {
        let mut catalog = Self::default();
        let mut defaults = BTreeMap::<DefaultKey, BTreeMap<String, Vec<SyntaxGuard>>>::new();
        let mut implementations = Vec::new();
        let mut resolver = routes::Resolver::new(bindings, budget);
        for file in &index.files {
            for fact in &file.associated_items {
                catalog.collect_fact(
                    fact,
                    &file.relative,
                    file.syntax,
                    &mut resolver,
                    &mut defaults,
                    &mut implementations,
                )?;
            }
        }
        for implementation in implementations {
            let key = DefaultKey {
                domain: implementation.domain.clone(),
                trait_name: implementation.trait_name.clone(),
            };
            let Some(items) = defaults.get(&key) else {
                continue;
            };
            for (item, guards) in items {
                for guard in guards {
                    catalog.insert(
                        implementation.domain.clone(),
                        implementation.self_type.clone(),
                        item.clone(),
                        TraitIdentity::Canonical(implementation.trait_name.clone()),
                        implementation.guard.combine(guard.clone()),
                        implementation.external_self,
                    );
                }
            }
        }
        Ok(catalog)
    }

    pub(super) fn classify_value(
        &self,
        route: &mut Route,
        context: &SyntaxGuard,
        selection: super::qualification::TraitSelection<'_>,
    ) {
        classification::classify(self, route, context, selection);
    }

    fn collect_fact(
        &mut self,
        fact: &AssociatedItemFact,
        file: &str,
        syntax: SourceSyntax,
        resolver: &mut routes::Resolver<'_>,
        defaults: &mut BTreeMap<DefaultKey, BTreeMap<String, Vec<SyntaxGuard>>>,
        implementations: &mut Vec<DefaultImpl>,
    ) -> Result<(), ProjectionLimit> {
        match &fact.kind {
            AssociatedItemKind::TraitDefault { trait_path, item } => {
                for (domain, route) in
                    resolver.local_trait_routes(fact, trait_path, file, syntax)?
                {
                    defaults
                        .entry(DefaultKey {
                            domain,
                            trait_name: route.name,
                        })
                        .or_default()
                        .entry(item.clone())
                        .or_default()
                        .push(fact.guard.clone());
                }
            }
            AssociatedItemKind::Implementation {
                self_type,
                trait_path,
                item,
            } => {
                let self_routes = resolver.self_routes(fact, self_type, file, syntax)?;
                let traits = resolver.trait_routes(fact, trait_path.as_deref(), file, syntax)?;
                for route in self_routes {
                    let (identity, origin) =
                        trait_for_domain(trait_path.as_deref(), &traits, &route.domain);
                    if route.origin == ResolvedOrigin::External
                        && origin != Some(ResolvedOrigin::CrateLocal)
                    {
                        continue;
                    }
                    if let Some(item) = item {
                        let external_self = route.origin == ResolvedOrigin::External;
                        self.insert(
                            route.domain,
                            route.name,
                            item.clone(),
                            identity,
                            fact.guard.clone(),
                            external_self,
                        );
                    } else if let TraitIdentity::Canonical(trait_name) = identity
                        && origin == Some(ResolvedOrigin::CrateLocal)
                    {
                        implementations.push(DefaultImpl {
                            domain: route.domain,
                            self_type: route.name,
                            trait_name,
                            guard: fact.guard.clone(),
                            external_self: route.origin == ResolvedOrigin::External,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn insert(
        &mut self,
        domain: CompilationDomain,
        self_type: String,
        item: String,
        trait_identity: TraitIdentity,
        guard: SyntaxGuard,
        external_self: bool,
    ) {
        let key = Key {
            domain,
            self_type,
            item,
        };
        if external_self {
            self.external_self.insert(key.clone());
        }
        self.entries
            .entry(key)
            .or_default()
            .entry(trait_identity)
            .or_default()
            .push(guard);
    }
}

fn trait_for_domain(
    raw: Option<&str>,
    routes: &BTreeMap<CompilationDomain, Vec<routes::TraitRoute>>,
    domain: &CompilationDomain,
) -> (TraitIdentity, Option<ResolvedOrigin>) {
    let Some(raw) = raw else {
        return (TraitIdentity::Inherent, None);
    };
    let Some([route]) = routes.get(domain).map(Vec::as_slice) else {
        return (TraitIdentity::Unresolved(raw.into()), None);
    };
    (
        TraitIdentity::Canonical(route.name.clone()),
        Some(route.origin),
    )
}
