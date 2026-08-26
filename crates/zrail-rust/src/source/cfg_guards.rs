//! Canonical cfg predicates separate exact features from conservative target atoms.

use syn::{Attribute, Meta, Token, punctuated::Punctuated};

use super::{CfgPredicate, SyntaxGuard};

#[derive(Clone)]
pub(in crate::source) struct GuardedAttributeEffect {
    pub(in crate::source) meta: Meta,
    pub(in crate::source) guard: SyntaxGuard,
}

#[cfg(test)]
pub(in crate::source) fn is_cfg_test(attribute: &Attribute) -> bool {
    cfg_meta(attribute).is_some_and(|meta| {
        CfgPredicate::from_meta(&meta).evaluate(&super::CfgContext {
            test: false,
            active_features: None,
        }) == super::CfgTruth::False
    })
}

pub(in crate::source) fn cfg_guard(attributes: &[Attribute]) -> SyntaxGuard {
    let predicates = attributes
        .iter()
        .filter_map(attribute_presence_predicate)
        .collect::<Vec<_>>();
    SyntaxGuard::from_predicate(CfgPredicate::all(predicates))
}

pub(in crate::source) fn feature_cfg_attr_requires_completeness(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        if !attribute.path().is_ident("cfg_attr") {
            return false;
        }
        guarded_attribute_effects(attribute).is_ok_and(|effects| {
            effects.iter().any(|effect| {
                contains_feature(&effect.guard.predicate())
                    && matches_effect(&effect.meta, &["path", "test", "bench"])
            })
        })
    })
}

fn attribute_presence_predicate(attribute: &Attribute) -> Option<CfgPredicate> {
    if !attribute.path().is_ident("cfg") && !attribute.path().is_ident("cfg_attr") {
        return None;
    }
    let Ok(effects) = guarded_attribute_effects(attribute) else {
        return Some(CfgPredicate::Opaque("malformed-cfg-attribute".into()));
    };
    Some(CfgPredicate::all(
        effects
            .into_iter()
            .filter(|effect| effect.meta.path().is_ident("cfg"))
            .map(|effect| {
                let predicate = cfg_effect_predicate(&effect.meta);
                CfgPredicate::any(vec![CfgPredicate::not(effect.guard.predicate()), predicate])
            })
            .collect(),
    ))
}

#[cfg(test)]
fn cfg_meta(attribute: &Attribute) -> Option<Meta> {
    let Meta::List(list) = &attribute.meta else {
        return None;
    };
    list.path
        .is_ident("cfg")
        .then(|| syn::parse2::<Meta>(list.tokens.clone()).ok())
        .flatten()
}

pub(in crate::source) fn guarded_attribute_effects(
    attribute: &Attribute,
) -> Result<Vec<GuardedAttributeEffect>, ()> {
    let mut effects = Vec::new();
    collect_effects(&attribute.meta, &SyntaxGuard::Ordinary, &mut effects)?;
    Ok(effects)
}

fn collect_effects(
    meta: &Meta,
    guard: &SyntaxGuard,
    effects: &mut Vec<GuardedAttributeEffect>,
) -> Result<(), ()> {
    if !meta.path().is_ident("cfg_attr") {
        effects.push(GuardedAttributeEffect {
            meta: meta.clone(),
            guard: guard.clone(),
        });
        return Ok(());
    }
    let Meta::List(list) = meta else {
        return Err(());
    };
    let arguments = list
        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        .map_err(|_| ())?;
    if arguments.len() < 2 {
        return Err(());
    }
    let Some(condition) = arguments.first() else {
        return Err(());
    };
    let conditional = guard.combine(SyntaxGuard::from_predicate(CfgPredicate::from_meta(
        condition,
    )));
    for nested in arguments.iter().skip(1) {
        collect_effects(nested, &conditional, effects)?;
    }
    Ok(())
}

fn cfg_effect_predicate(meta: &Meta) -> CfgPredicate {
    let Meta::List(list) = meta else {
        return CfgPredicate::Opaque("malformed-cfg".into());
    };
    syn::parse2::<Meta>(list.tokens.clone()).map_or_else(
        |_| CfgPredicate::Opaque("malformed-cfg".into()),
        |predicate| CfgPredicate::from_meta(&predicate),
    )
}

fn matches_effect(meta: &Meta, names: &[&str]) -> bool {
    names.iter().any(|name| meta.path().is_ident(name))
}

fn contains_feature(predicate: &CfgPredicate) -> bool {
    match predicate {
        CfgPredicate::Feature(_) => true,
        CfgPredicate::Not(value) => contains_feature(value),
        CfgPredicate::All(values) | CfgPredicate::Any(values) => {
            values.iter().any(contains_feature)
        }
        CfgPredicate::True | CfgPredicate::False | CfgPredicate::Test | CfgPredicate::Opaque(_) => {
            false
        }
    }
}

#[cfg(test)]
#[path = "cfg_guards_test.rs"]
mod cfg_guards_test;
