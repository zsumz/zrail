//! Syntax helpers for type-policy facts keep the collector traversal small.

use proc_macro2::TokenTree;
use syn::{Type, UseTree, spanned::Spanned};
use zrail_core::{DuplicationTrait, SourceSpan};

use super::{
    SyntaxGuard,
    fact::source_span,
    type_policy_model::{
        DerivedTraitFact, DuplicationSyntaxFact, DuplicationSyntaxKind, TypeFieldFact,
    },
};

pub(super) fn named_fields(
    fields: &syn::Fields,
    enclosing: &SyntaxGuard,
) -> Option<Vec<TypeFieldFact>> {
    let syn::Fields::Named(fields) = fields else {
        return None;
    };
    Some(
        fields
            .named
            .iter()
            .filter_map(|field| {
                Some(TypeFieldFact {
                    name: field.ident.as_ref()?.to_string(),
                    type_shape: super::type_shape::type_shape(&field.ty),
                    visibility: visibility(&field.vis),
                    guard: enclosing.combine(super::attributes::cfg_guard(&field.attrs)),
                })
            })
            .collect(),
    )
}

pub(super) fn nominal_type_span(ty: &Type) -> Option<SourceSpan> {
    let Type::Path(path) = ty else {
        return None;
    };
    path.qself.is_none().then(|| source_span(path.path.span()))
}

pub(super) fn derives(
    attributes: &[syn::Attribute],
    enclosing: &SyntaxGuard,
) -> Vec<DerivedTraitFact> {
    attributes
        .iter()
        .flat_map(|attribute| {
            super::macro_expansion::attribute_paths(attribute).unwrap_or_default()
        })
        .filter(|expansion| expansion.kind == super::macro_expansion::ExpansionKind::Derive)
        .map(|expansion| DerivedTraitFact {
            trait_hint: last_segment(&expansion.path),
            span: source_span(expansion.path.span()),
            guard: enclosing.combine(expansion.guard),
        })
        .collect()
}

pub(super) fn collect_use(
    tree: &UseTree,
    guard: &SyntaxGuard,
    scope: &[SourceSpan],
    facts: &mut Vec<DuplicationSyntaxFact>,
) {
    match tree {
        UseTree::Path(path) => collect_use(&path.tree, guard, scope, facts),
        UseTree::Name(name) => push_syntax(&name.ident, guard, scope, facts),
        UseTree::Rename(rename) => {
            push_syntax(&rename.ident, guard, scope, facts);
            push_syntax(&rename.rename, guard, scope, facts);
        }
        UseTree::Group(group) => {
            for tree in &group.items {
                collect_use(tree, guard, scope, facts);
            }
        }
        UseTree::Glob(_) => {}
    }
}

pub(super) fn collect_tokens(
    tokens: proc_macro2::TokenStream,
    guard: &SyntaxGuard,
    scope: &[SourceSpan],
    facts: &mut Vec<DuplicationSyntaxFact>,
) {
    for token in tokens {
        match token {
            TokenTree::Ident(ident) => {
                push_token(&ident, guard, scope, facts);
            }
            TokenTree::Group(group) => collect_tokens(group.stream(), guard, scope, facts),
            TokenTree::Literal(_) | TokenTree::Punct(_) => {}
        }
    }
}

fn push_syntax(
    ident: &syn::Ident,
    guard: &SyntaxGuard,
    scope: &[SourceSpan],
    facts: &mut Vec<DuplicationSyntaxFact>,
) {
    push(ident, DuplicationSyntaxKind::Import, guard, scope, facts);
}

fn push_token(
    ident: &syn::Ident,
    guard: &SyntaxGuard,
    scope: &[SourceSpan],
    facts: &mut Vec<DuplicationSyntaxFact>,
) {
    push(
        ident,
        DuplicationSyntaxKind::MacroToken,
        guard,
        scope,
        facts,
    );
}

fn push(
    ident: &syn::Ident,
    kind: DuplicationSyntaxKind,
    guard: &SyntaxGuard,
    scope: &[SourceSpan],
    facts: &mut Vec<DuplicationSyntaxFact>,
) {
    if let Some(trait_name) = duplication_trait(&ident.to_string()) {
        facts.push(DuplicationSyntaxFact {
            kind,
            trait_name,
            span: source_span(ident.span()),
            guard: guard.clone(),
            lexical_scope: scope.to_vec(),
        });
    }
}

pub(super) fn duplication_trait(name: &str) -> Option<DuplicationTrait> {
    match name {
        "Clone" => Some(DuplicationTrait::Clone),
        "Copy" => Some(DuplicationTrait::Copy),
        _ => None,
    }
}

pub(super) fn last_segment(path: &syn::Path) -> String {
    path.segments
        .last()
        .map_or_else(String::new, |segment| segment.ident.to_string())
}

pub(super) fn visibility(value: &syn::Visibility) -> String {
    match value {
        syn::Visibility::Inherited => "private".into(),
        syn::Visibility::Public(_) => "pub".into(),
        syn::Visibility::Restricted(restricted) => {
            let path = restricted
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            if restricted.in_token.is_some() {
                format!("pub(in {path})")
            } else {
                format!("pub({path})")
            }
        }
    }
}
