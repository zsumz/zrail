//! Inline associated constraints become projection-specific facts.

use syn::{GenericArgument, TypeParamBound};
use zrail_core::{AnalysisQuality, SourceSpan};

use super::super::{
    AssociatedSegment, BoundSubject, GenericArgumentsIdentity, GenericPathIdentity,
    ProjectionIdentity, SyntaxGuard, TraitBoundFact, fact::source_span,
};

pub(super) fn from_bounds(
    subject: &BoundSubject,
    bounds: &syn::punctuated::Punctuated<TypeParamBound, syn::Token![+]>,
    guard: &SyntaxGuard,
    scope: &[SourceSpan],
    span: SourceSpan,
) -> Vec<TraitBoundFact> {
    let mut base = TraitBoundFact {
        subject: subject.clone(),
        providers: Vec::new(),
        equalities: Vec::new(),
        quality: AnalysisQuality::Exact,
        guard: guard.clone(),
        lexical_scope: scope.to_vec(),
        span,
    };
    let mut associated = Vec::new();
    for bound in bounds {
        let TypeParamBound::Trait(bound) = bound else {
            continue;
        };
        if !matches!(bound.modifier, syn::TraitBoundModifier::None) {
            continue;
        }
        let qualifier = GenericPathIdentity::trait_path(&bound.path);
        base.providers.push(qualifier.clone());
        base.quality = base.quality.max(qualifier.quality());
        associated.extend(path_constraints(
            subject,
            &qualifier,
            &bound.path,
            guard,
            scope,
        ));
    }
    if !base.providers.is_empty() {
        associated.push(base);
    }
    associated
}

fn path_constraints(
    subject: &BoundSubject,
    qualifier: &GenericPathIdentity,
    path: &syn::Path,
    guard: &SyntaxGuard,
    scope: &[SourceSpan],
) -> Vec<TraitBoundFact> {
    path.segments
        .iter()
        .flat_map(|segment| match &segment.arguments {
            syn::PathArguments::AngleBracketed(arguments) => arguments
                .args
                .iter()
                .filter_map(|argument| constraint(subject, qualifier, argument, guard, scope))
                .collect(),
            syn::PathArguments::None | syn::PathArguments::Parenthesized(_) => Vec::new(),
        })
        .collect()
}

fn constraint(
    subject: &BoundSubject,
    qualifier: &GenericPathIdentity,
    argument: &GenericArgument,
    guard: &SyntaxGuard,
    scope: &[SourceSpan],
) -> Option<TraitBoundFact> {
    let (segment, providers, target, span) = match argument {
        GenericArgument::Constraint(value) => (
            AssociatedSegment::constrained(&value.ident, value.generics.as_ref()),
            value.bounds.iter().filter_map(provider).collect(),
            None,
            source_span(value.ident.span()),
        ),
        GenericArgument::AssocType(value) => (
            AssociatedSegment::constrained(&value.ident, value.generics.as_ref()),
            Vec::new(),
            GenericPathIdentity::type_path(&value.ty),
            source_span(value.ident.span()),
        ),
        GenericArgument::Lifetime(_)
        | GenericArgument::Type(_)
        | GenericArgument::Const(_)
        | GenericArgument::AssocConst(_)
        | _ => return None,
    };
    let mut quality = qualifier.quality().max(segment_quality(&segment));
    if matches!(argument, GenericArgument::AssocType(_)) && target.is_none() {
        quality = AnalysisQuality::Unresolved;
    }
    Some(TraitBoundFact {
        subject: project(subject, qualifier.clone(), segment),
        providers,
        equalities: target.into_iter().collect(),
        quality,
        guard: guard.clone(),
        lexical_scope: scope.to_vec(),
        span,
    })
}

fn project(
    subject: &BoundSubject,
    qualifier: GenericPathIdentity,
    associated: AssociatedSegment,
) -> BoundSubject {
    let root = subject.root().to_owned();
    let mut projection = subject.projection().cloned().unwrap_or_default();
    if projection.qualifying_trait.is_none() {
        projection.qualifying_trait = Some(qualifier);
    } else {
        projection.qualifying_trait = Some(GenericPathIdentity {
            path: "<unresolved nested projection>".into(),
            arguments: GenericArgumentsIdentity::Unknown,
        });
    }
    projection.associated.push(associated);
    BoundSubject::Projection { root, projection }
}

fn provider(bound: &TypeParamBound) -> Option<GenericPathIdentity> {
    match bound {
        TypeParamBound::Trait(bound) if matches!(bound.modifier, syn::TraitBoundModifier::None) => {
            Some(GenericPathIdentity::trait_path(&bound.path))
        }
        _ => None,
    }
}

fn segment_quality(segment: &AssociatedSegment) -> AnalysisQuality {
    ProjectionIdentity {
        qualifying_trait: None,
        associated: vec![segment.clone()],
    }
    .quality()
}
