//! Attribute predicates used by hygiene and test-placement indexing.

use syn::{AttrStyle, Attribute, Expr, ExprLit, Lit, Meta, Token, punctuated::Punctuated};

#[cfg(test)]
pub(super) use super::cfg_guards::is_cfg_test;
pub(super) use super::cfg_guards::{cfg_conditions_are_exact, cfg_guard};

const UNSAFE_ATTRIBUTES: [&str; 4] = ["export_name", "link_section", "naked", "no_mangle"];

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

pub(super) fn is_lint_suppression(attribute: &Attribute) -> bool {
    lint_suppression_reason(&attribute.meta).is_some()
}

pub(super) fn unsafe_attribute_names(attribute: &Attribute) -> Vec<&'static str> {
    let mut names = Vec::new();
    collect_unsafe_attributes(&attribute.meta, &mut names);
    names.sort_unstable();
    names.dedup();
    names
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
    if list.path.is_ident("unsafe") {
        if let Ok(nested) = syn::parse2::<Meta>(list.tokens.clone()) {
            collect_unsafe_attributes(&nested, names);
        }
    } else if list.path.is_ident("cfg_attr")
        && let Some(arguments) = cfg_arguments(list)
    {
        for nested in arguments.iter().skip(1) {
            collect_unsafe_attributes(nested, names);
        }
    }
}

pub(super) fn lint_suppression_is_reasoned(attribute: &Attribute) -> bool {
    lint_suppression_reason(&attribute.meta).is_some_and(|reasoned| reasoned)
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
    let Meta::List(list) = meta else {
        return None;
    };
    if !list.path.is_ident("cfg_attr") {
        return None;
    }
    let arguments = cfg_arguments(list)?;
    let suppressions = arguments
        .iter()
        .skip(1)
        .filter_map(lint_suppression_reason)
        .collect::<Vec<_>>();
    (!suppressions.is_empty()).then(|| suppressions.into_iter().all(|reasoned| reasoned))
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
