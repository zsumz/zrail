//! Attribute predicates used by hygiene and test-placement indexing.

use syn::{AttrStyle, Attribute, Expr, ExprLit, Lit, Meta, Token, punctuated::Punctuated};

use super::SyntaxGuard;

#[cfg(test)]
pub(super) use super::cfg::cfg_guards::is_cfg_test;
pub(super) use super::cfg::cfg_guards::{cfg_guard, feature_cfg_attr_requires_completeness};

const UNSAFE_ATTRIBUTES: [&str; 4] = ["export_name", "link_section", "naked", "no_mangle"];

pub(super) struct GuardedLintSuppression {
    pub(super) reasoned: bool,
    pub(super) guard: SyntaxGuard,
}

pub(super) struct GuardedUnsafeAttribute {
    pub(super) name: &'static str,
    pub(super) guard: SyntaxGuard,
}

pub(super) fn has_module_docs(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        matches!(attribute.style, AttrStyle::Inner(_)) && attribute.path().is_ident("doc")
    })
}

pub(super) fn path_attribute(attributes: &[Attribute]) -> Option<String> {
    let mut matches = attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("path"));
    let attribute = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    let Meta::NameValue(value) = &attribute.meta else {
        return None;
    };
    let Expr::Lit(ExprLit {
        lit: Lit::Str(path),
        ..
    }) = &value.value
    else {
        return None;
    };
    Some(path.value())
}

pub(super) fn has_path_attribute(attributes: &[Attribute]) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.path().is_ident("path"))
}

pub(super) fn has_conditional_path_attribute(attributes: &[Attribute]) -> bool {
    attributes
        .iter()
        .any(|attribute| cfg_attr_mentions(attribute, "path"))
}

pub(super) fn is_test_attribute(attribute: &Attribute) -> bool {
    cfg_attr_mentions(attribute, "test")
        || attribute
            .path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "test")
}

#[cfg(test)]
pub(super) fn is_lint_suppression(attribute: &Attribute) -> bool {
    !lint_suppression_effects(attribute).is_empty()
}

pub(super) fn unsafe_attribute_effects(attribute: &Attribute) -> Vec<GuardedUnsafeAttribute> {
    let Ok(effects) = super::cfg::cfg_guards::guarded_attribute_effects(attribute) else {
        return Vec::new();
    };
    effects
        .into_iter()
        .flat_map(|effect| {
            let mut names = Vec::new();
            collect_unsafe_attributes(&effect.meta, &mut names);
            names.into_iter().map(move |name| GuardedUnsafeAttribute {
                name,
                guard: effect.guard.clone(),
            })
        })
        .collect()
}

fn collect_unsafe_attributes(meta: &Meta, names: &mut Vec<&'static str>) {
    if let Some(name) = UNSAFE_ATTRIBUTES
        .iter()
        .find(|name| meta.path().is_ident(name))
    {
        names.push(name);
        return;
    }
    let Meta::List(list) = meta else {
        return;
    };
    if list.path.is_ident("unsafe")
        && let Ok(nested) = syn::parse2::<Meta>(list.tokens.clone())
    {
        collect_unsafe_attributes(&nested, names);
    }
}

#[cfg(test)]
pub(super) fn lint_suppression_is_reasoned(attribute: &Attribute) -> bool {
    let effects = lint_suppression_effects(attribute);
    !effects.is_empty() && effects.iter().all(|effect| effect.reasoned)
}

pub(super) fn lint_suppression_effects(attribute: &Attribute) -> Vec<GuardedLintSuppression> {
    let Ok(effects) = super::cfg::cfg_guards::guarded_attribute_effects(attribute) else {
        return Vec::new();
    };
    effects
        .into_iter()
        .filter_map(|effect| {
            lint_suppression_reason(&effect.meta).map(|reasoned| GuardedLintSuppression {
                reasoned,
                guard: effect.guard,
            })
        })
        .collect()
}

fn lint_suppression_reason(meta: &Meta) -> Option<bool> {
    if meta.path().is_ident("allow") || meta.path().is_ident("expect") {
        let Meta::List(list) = meta else {
            return Some(false);
        };
        let Some(arguments) = cfg_arguments(list) else {
            return Some(false);
        };
        return Some(arguments.iter().any(nonempty_reason));
    }
    None
}

fn nonempty_reason(meta: &Meta) -> bool {
    let Meta::NameValue(value) = meta else {
        return false;
    };
    if !value.path.is_ident("reason") {
        return false;
    }
    matches!(
        &value.value,
        Expr::Lit(ExprLit { lit: Lit::Str(reason), .. }) if !reason.value().trim().is_empty()
    )
}

fn cfg_arguments(list: &syn::MetaList) -> Option<Punctuated<Meta, Token![,]>> {
    list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        .ok()
}

fn cfg_attr_mentions(attribute: &Attribute, name: &str) -> bool {
    let Meta::List(list) = &attribute.meta else {
        return false;
    };
    if !list.path.is_ident("cfg_attr") {
        return false;
    }
    let Ok(arguments) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
    else {
        return false;
    };
    arguments
        .iter()
        .skip(1)
        .any(|meta| meta_mentions(meta, name))
}

fn meta_mentions(meta: &Meta, name: &str) -> bool {
    if meta.path().is_ident(name) {
        return true;
    }
    let Meta::List(list) = meta else {
        return false;
    };
    if !list.path.is_ident("cfg_attr") {
        return false;
    }
    let Ok(arguments) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
    else {
        return false;
    };
    arguments
        .iter()
        .skip(1)
        .any(|nested| meta_mentions(nested, name))
}

#[cfg(test)]
#[path = "attributes_test.rs"]
mod attributes_test;
