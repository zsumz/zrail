//! Syntax facts used by exact type-shape and duplication rails.

use zrail_core::{DuplicationTrait, SourceSpan};

use super::SyntaxGuard;

#[derive(Clone, Debug, Default)]
pub(crate) struct TypePolicyFacts {
    pub(crate) declarations: Vec<TypeDeclarationFact>,
    pub(crate) trait_impls: Vec<TraitImplFact>,
    pub(crate) syntax: Vec<DuplicationSyntaxFact>,
}

#[derive(Clone, Debug)]
pub(crate) struct TypeDeclarationFact {
    pub(crate) identity_span: SourceSpan,
    pub(crate) kind: TypeDeclarationKind,
    pub(crate) visibility: String,
    pub(crate) fields: Option<Vec<TypeFieldFact>>,
    pub(crate) derives: Vec<DerivedTraitFact>,
    pub(crate) guard: SyntaxGuard,
    pub(crate) lexical_scope: Vec<SourceSpan>,
    pub(crate) leaf_module: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TypeDeclarationKind {
    NamedStruct,
    Other,
}

#[derive(Clone, Debug)]
pub(crate) struct TypeFieldFact {
    pub(crate) name: String,
    pub(crate) type_shape: super::type_shape::TypeShapeFact,
    pub(crate) visibility: String,
    pub(crate) guard: SyntaxGuard,
}

#[derive(Clone, Debug)]
pub(crate) struct DerivedTraitFact {
    pub(crate) trait_hint: String,
    pub(crate) span: SourceSpan,
    pub(crate) guard: SyntaxGuard,
}

#[derive(Clone, Debug)]
pub(crate) struct TraitImplFact {
    pub(crate) trait_span: SourceSpan,
    pub(crate) trait_hint: String,
    pub(crate) type_span: Option<SourceSpan>,
    pub(crate) polarity: TraitImplPolarity,
    pub(crate) guard: SyntaxGuard,
    pub(crate) lexical_scope: Vec<SourceSpan>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TraitImplPolarity {
    Positive,
    Negative,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DuplicationSyntaxKind {
    Import,
    MacroToken,
}

#[derive(Clone, Debug)]
pub(crate) struct DuplicationSyntaxFact {
    pub(crate) kind: DuplicationSyntaxKind,
    pub(crate) trait_name: DuplicationTrait,
    pub(crate) span: SourceSpan,
    pub(crate) guard: SyntaxGuard,
    pub(crate) lexical_scope: Vec<SourceSpan>,
}
