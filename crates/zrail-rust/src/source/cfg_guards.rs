//! Exact test-domain predicates separate `cfg(test)` from `cfg(not(test))`.

use syn::{Attribute, Meta, Token, punctuated::Punctuated};

use super::SyntaxGuard;

#[cfg(test)]
pub(super) fn is_cfg_test(attribute: &Attribute) -> bool {
    cfg_meta(attribute).is_some_and(|meta| cfg_implies_test(&meta))
}

pub(super) fn cfg_guard(attributes: &[Attribute]) -> SyntaxGuard {
    attributes
        .iter()
        .fold(SyntaxGuard::Ordinary, |guard, attribute| {
            if attribute.path().is_ident("cfg_attr") {
                return guard.combine(SyntaxGuard::Conditional);
            }
            if !attribute.path().is_ident("cfg") {
                return guard;
            }
            let Some(meta) = cfg_meta(attribute) else {
                return guard.combine(SyntaxGuard::Conditional);
            };
            let nested = if cfg_implies_test(&meta) {
                SyntaxGuard::TestOnly
            } else if cfg_implies_not_test(&meta) {
                SyntaxGuard::ProductionOnly
            } else {
                SyntaxGuard::Ordinary
            };
            let nested = if cfg_meta_is_exact(&meta) {
                nested
            } else {
                nested.combine(SyntaxGuard::Conditional)
            };
            guard.combine(nested)
        })
}

pub(super) fn cfg_conditions_are_exact(attributes: &[Attribute]) -> bool {
    attributes.iter().all(|attribute| {
        if attribute.path().is_ident("cfg_attr") {
            return false;
        }
        if !attribute.path().is_ident("cfg") {
            return true;
        }
        cfg_meta(attribute).is_some_and(|meta| cfg_meta_is_exact(&meta))
    })
}

fn cfg_meta_is_exact(meta: &Meta) -> bool {
    matches!(meta, Meta::Path(path) if path.is_ident("test")) || exact_not_test(meta)
}

fn cfg_meta(attribute: &Attribute) -> Option<Meta> {
    let Meta::List(list) = &attribute.meta else {
        return None;
    };
    list.path
        .is_ident("cfg")
        .then(|| syn::parse2::<Meta>(list.tokens.clone()).ok())
        .flatten()
}

fn cfg_implies_test(meta: &Meta) -> bool {
    match meta {
        Meta::Path(path) => path.is_ident("test"),
        Meta::List(list) if list.path.is_ident("all") => {
            cfg_arguments(list).is_some_and(|arguments| arguments.iter().any(cfg_implies_test))
        }
        Meta::List(list) if list.path.is_ident("any") => {
            cfg_arguments(list).is_some_and(|arguments| {
                !arguments.is_empty() && arguments.iter().all(cfg_implies_test)
            })
        }
        Meta::List(_) | Meta::NameValue(_) => false,
    }
}

fn cfg_implies_not_test(meta: &Meta) -> bool {
    match meta {
        Meta::List(list) if list.path.is_ident("not") => exact_not_test(meta),
        Meta::List(list) if list.path.is_ident("all") => {
            cfg_arguments(list).is_some_and(|arguments| arguments.iter().any(cfg_implies_not_test))
        }
        Meta::List(list) if list.path.is_ident("any") => {
            cfg_arguments(list).is_some_and(|arguments| {
                !arguments.is_empty() && arguments.iter().all(cfg_implies_not_test)
            })
        }
        Meta::Path(_) | Meta::List(_) | Meta::NameValue(_) => false,
    }
}

fn exact_not_test(meta: &Meta) -> bool {
    let Meta::List(list) = meta else {
        return false;
    };
    if !list.path.is_ident("not") {
        return false;
    }
    cfg_arguments(list).is_some_and(|arguments| {
        arguments.len() == 1
            && matches!(arguments.first(), Some(Meta::Path(path)) if path.is_ident("test"))
    })
}

fn cfg_arguments(list: &syn::MetaList) -> Option<Punctuated<Meta, Token![,]>> {
    list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        .ok()
}
