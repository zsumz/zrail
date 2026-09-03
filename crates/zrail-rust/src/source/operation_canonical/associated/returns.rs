//! Associated return shapes prove local value types only across exact worlds.

use std::collections::{BTreeMap, BTreeSet};

use crate::source::{
    AssociatedItemFact, CfgPredicate, CompilationDomain, GuardAvailability, SourceSyntax,
    SyntaxGuard, include_bindings::ResolvedTerminal, include_projection_budget::ProjectionLimit,
};

use super::{Catalog, Key, Route, routes};

pub(super) enum Depths {
    Absent,
    Direct,
    Wrapped {
        depth: usize,
        domains: BTreeSet<CompilationDomain>,
    },
}

impl Depths {
    pub(super) fn for_domain(&self, domain: &CompilationDomain) -> Option<usize> {
        match self {
            Self::Absent => None,
            Self::Direct => Some(0),
            Self::Wrapped { depth, domains } => domains.contains(domain).then_some(*depth),
        }
    }
}

pub(super) fn resolve(
    fact: &AssociatedItemFact,
    file: &str,
    syntax: SourceSyntax,
    resolver: &mut routes::Resolver<'_>,
) -> Result<Depths, ProjectionLimit> {
    let Some(shape) = &fact.return_shape else {
        return Ok(Depths::Absent);
    };
    if shape.wrappers.is_empty() {
        return Ok(Depths::Direct);
    }
    let mut supported = None::<BTreeSet<CompilationDomain>>;
    for wrapper in &shape.wrappers {
        let routes = resolver.type_routes(fact, wrapper, file, syntax)?;
        let mut grouped = BTreeMap::<CompilationDomain, Vec<String>>::new();
        for route in routes {
            grouped.entry(route.domain).or_default().push(route.name);
        }
        let exact = grouped
            .into_iter()
            .filter_map(|(domain, mut names)| {
                names.sort();
                names.dedup();
                matches!(names.as_slice(), [name] if is_builtin_try(name)).then_some(domain)
            })
            .collect::<BTreeSet<_>>();
        supported = Some(match supported {
            None => exact,
            Some(current) => current.intersection(&exact).cloned().collect(),
        });
    }
    Ok(Depths::Wrapped {
        depth: shape.wrappers.len(),
        domains: supported.unwrap_or_default(),
    })
}

pub(super) fn matches(
    catalog: &Catalog,
    route: &Route,
    context: &SyntaxGuard,
    try_depth: usize,
) -> bool {
    if route.terminal != ResolvedTerminal::Value {
        return false;
    }
    let Some((self_type, item)) = route.name.rsplit_once("::") else {
        return false;
    };
    let key = Key {
        domain: route.domain.clone(),
        self_type: self_type.into(),
        item: item.into(),
    };
    if catalog.external_self.contains(&key) {
        return false;
    }
    let Some(traits) = catalog.entries.get(&key) else {
        return false;
    };
    let declarations = traits
        .values()
        .flatten()
        .filter(|declaration| {
            declaration
                .guard
                .availability_for_domain(context, &route.domain)
                != GuardAvailability::Absent
        })
        .collect::<Vec<_>>();
    if declarations.is_empty()
        || declarations
            .iter()
            .any(|declaration| declaration.return_try_depth != Some(try_depth))
    {
        return false;
    }
    let union = SyntaxGuard::from_predicate(CfgPredicate::any(
        declarations
            .iter()
            .map(|declaration| declaration.guard.predicate())
            .collect(),
    ));
    union.availability_for_domain(context, &route.domain) == GuardAvailability::Exact
}

fn is_builtin_try(name: &str) -> bool {
    matches!(
        name,
        "core::option::Option"
            | "core::result::Result"
            | "std::option::Option"
            | "std::result::Result"
    )
}
