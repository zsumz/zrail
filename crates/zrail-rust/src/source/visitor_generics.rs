//! Generic scopes retain the traits that can provide associated identities.

use std::collections::{BTreeMap, BTreeSet};

use syn::{GenericParam, Generics, Type, TypeParamBound, WherePredicate};
use zrail_core::AnalysisQuality;

use super::super::{GenericAssociatedCandidate, GenericParameterBounds, fact::written_path};
use super::FactVisitor;

#[derive(Debug)]
pub(in crate::source) struct GenericBoundScope {
    declared: BTreeSet<String>,
    bounds: BTreeMap<String, Vec<String>>,
}

impl FactVisitor<'_> {
    pub(in crate::source) fn with_generics(
        &mut self,
        generics: &Generics,
        include_self: bool,
        visit: impl FnOnce(&mut Self),
    ) {
        let checkpoint = (self.generic_types.len(), self.generic_values.len());
        if include_self {
            self.generic_types.push("Self".into());
        }
        self.generic_types.extend(
            generics
                .type_params()
                .map(|parameter| parameter.ident.to_string()),
        );
        self.generic_values.extend(
            generics
                .const_params()
                .map(|parameter| parameter.ident.to_string()),
        );
        self.generic_bound_scopes.push(scope(generics));
        visit(self);
        self.generic_bound_scopes.pop();
        self.generic_types.truncate(checkpoint.0);
        self.generic_values.truncate(checkpoint.1);
    }

    pub(in crate::source) fn with_fresh_generics(&mut self, visit: impl FnOnce(&mut Self)) {
        let inherited_types = std::mem::take(&mut self.generic_types);
        let inherited_values = std::mem::take(&mut self.generic_values);
        let inherited_bounds = std::mem::take(&mut self.generic_bound_scopes);
        visit(self);
        self.generic_types = inherited_types;
        self.generic_values = inherited_values;
        self.generic_bound_scopes = inherited_bounds;
    }

    pub(in crate::source) fn active_generic_bounds(&self) -> Vec<GenericParameterBounds> {
        let mut effective = BTreeMap::<String, Vec<String>>::new();
        for scope in &self.generic_bound_scopes {
            for parameter in &scope.declared {
                effective.insert(parameter.clone(), Vec::new());
            }
            for (parameter, traits) in &scope.bounds {
                let entry = effective.entry(parameter.clone()).or_default();
                entry.extend(traits.iter().cloned());
                entry.sort();
                entry.dedup();
            }
        }
        effective
            .into_iter()
            .map(|(parameter, traits)| GenericParameterBounds { parameter, traits })
            .collect()
    }

    pub(in crate::source) fn generic_associated_candidates(
        &self,
        written: &str,
    ) -> Vec<GenericAssociatedCandidate> {
        let Some((root, suffix)) = written.split_once("::") else {
            return Vec::new();
        };
        self.active_generic_bounds()
            .into_iter()
            .find(|bounds| visible(&bounds.parameter) == visible(root))
            .into_iter()
            .flat_map(|bounds| bounds.traits)
            .map(|trait_path| GenericAssociatedCandidate {
                name: format!("{trait_path}::{suffix}"),
                canonical: Vec::new(),
                quality: AnalysisQuality::Exact,
            })
            .collect()
    }
}

fn scope(generics: &Generics) -> GenericBoundScope {
    let declared = generics
        .params
        .iter()
        .filter_map(|parameter| match parameter {
            GenericParam::Type(parameter) => Some(parameter.ident.to_string()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let mut bounds = BTreeMap::<String, Vec<String>>::new();
    for parameter in generics.type_params() {
        extend_bounds(&mut bounds, parameter.ident.to_string(), &parameter.bounds);
    }
    for predicate in generics
        .where_clause
        .iter()
        .flat_map(|clause| &clause.predicates)
    {
        let WherePredicate::Type(predicate) = predicate else {
            continue;
        };
        let Some(parameter) = bare_type_parameter(&predicate.bounded_ty) else {
            continue;
        };
        extend_bounds(&mut bounds, parameter, &predicate.bounds);
    }
    for traits in bounds.values_mut() {
        traits.sort();
        traits.dedup();
    }
    GenericBoundScope { declared, bounds }
}

fn extend_bounds(
    bounds: &mut BTreeMap<String, Vec<String>>,
    parameter: String,
    candidates: &syn::punctuated::Punctuated<TypeParamBound, syn::Token![+]>,
) {
    bounds
        .entry(parameter)
        .or_default()
        .extend(candidates.iter().filter_map(|bound| match bound {
            TypeParamBound::Trait(bound)
                if matches!(bound.modifier, syn::TraitBoundModifier::None) =>
            {
                Some(written_path(&bound.path))
            }
            _ => None,
        }));
}

fn bare_type_parameter(ty: &Type) -> Option<String> {
    let Type::Path(path) = ty else {
        return None;
    };
    (path.qself.is_none() && path.path.leading_colon.is_none() && path.path.segments.len() == 1)
        .then(|| path.path.segments[0].ident.to_string())
}

fn visible(name: &str) -> &str {
    name.strip_prefix("r#").unwrap_or(name)
}
