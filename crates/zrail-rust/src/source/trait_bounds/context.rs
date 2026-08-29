//! The active trait is retained separately from ordinary `Self` bounds.

use zrail_core::SourceSpan;

use super::super::{BoundSubject, GenericPathIdentity, SyntaxGuard, TraitBoundFact};

pub(in crate::source) fn current_trait_bounds(
    provider: GenericPathIdentity,
    guard: &SyntaxGuard,
    scope: &[SourceSpan],
    span: SourceSpan,
) -> Vec<TraitBoundFact> {
    let marker = GenericPathIdentity::current_trait_context();
    vec![
        super::explicit(
            BoundSubject::SelfType,
            vec![provider.clone()],
            guard,
            scope,
            span,
        ),
        super::explicit(
            BoundSubject::TypeParameter(marker.path),
            vec![provider],
            guard,
            scope,
            span,
        ),
    ]
}
