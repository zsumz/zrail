//! Canonical three-valued cfg predicates preserve feature and target identity.

use std::collections::BTreeSet;

use syn::{Expr, ExprLit, Lit, Meta};

use super::cfg_predicate_text::{arguments, canonical_meta, has_inverse, joined};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CfgTruth {
    False,
    True,
    Unknown,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CfgPredicate {
    True,
    False,
    Test,
    Feature(String),
    Opaque(String),
    Not(Box<Self>),
    All(Vec<Self>),
    Any(Vec<Self>),
}

pub(crate) struct CfgContext<'a> {
    pub(crate) test: bool,
    pub(crate) active_features: Option<&'a BTreeSet<String>>,
}

impl CfgPredicate {
    pub(crate) fn from_meta(meta: &Meta) -> Self {
        match meta {
            Meta::Path(path) if path.is_ident("test") => Self::Test,
            Meta::NameValue(value) if value.path.is_ident("feature") => match &value.value {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(feature),
                    ..
                }) => Self::Feature(feature.value()),
                _ => Self::opaque(meta),
            },
            Meta::List(list) if list.path.is_ident("all") => arguments(list).map_or_else(
                || Self::opaque(meta),
                |values| Self::all(values.iter().map(Self::from_meta).collect()),
            ),
            Meta::List(list) if list.path.is_ident("any") => arguments(list).map_or_else(
                || Self::opaque(meta),
                |values| Self::any(values.iter().map(Self::from_meta).collect()),
            ),
            Meta::List(list) if list.path.is_ident("not") => arguments(list)
                .filter(|values| values.len() == 1)
                .and_then(|values| values.first().map(Self::from_meta))
                .map_or_else(|| Self::opaque(meta), Self::not),
            _ => Self::opaque(meta),
        }
    }

    pub(crate) fn all(mut values: Vec<Self>) -> Self {
        let mut flattened = Vec::new();
        for value in values.drain(..) {
            match value {
                Self::False => return Self::False,
                Self::True => {}
                Self::All(nested) => flattened.extend(nested),
                value => flattened.push(value),
            }
        }
        flattened.sort();
        flattened.dedup();
        if has_inverse(&flattened) {
            return Self::False;
        }
        match flattened.len() {
            0 => Self::True,
            1 => flattened.pop().unwrap_or(Self::True),
            _ => Self::All(flattened),
        }
    }

    pub(crate) fn any(mut values: Vec<Self>) -> Self {
        let mut flattened = Vec::new();
        for value in values.drain(..) {
            match value {
                Self::True => return Self::True,
                Self::False => {}
                Self::Any(nested) => flattened.extend(nested),
                value => flattened.push(value),
            }
        }
        flattened.sort();
        flattened.dedup();
        if has_inverse(&flattened) {
            return Self::True;
        }
        match flattened.len() {
            0 => Self::False,
            1 => flattened.pop().unwrap_or(Self::False),
            _ => Self::Any(flattened),
        }
    }

    pub(crate) fn not(value: Self) -> Self {
        match value {
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Not(nested) => *nested,
            value => Self::Not(Box::new(value)),
        }
    }

    pub(crate) fn evaluate(&self, context: &CfgContext<'_>) -> CfgTruth {
        match self {
            Self::True => CfgTruth::True,
            Self::False => CfgTruth::False,
            Self::Test => truth(context.test),
            Self::Feature(feature) => context
                .active_features
                .map_or(CfgTruth::Unknown, |active| truth(active.contains(feature))),
            Self::Opaque(_) => CfgTruth::Unknown,
            Self::Not(value) => invert(value.evaluate(context)),
            Self::All(values) => values.iter().fold(CfgTruth::True, |left, right| {
                and(left, right.evaluate(context))
            }),
            Self::Any(values) => values.iter().fold(CfgTruth::False, |left, right| {
                or(left, right.evaluate(context))
            }),
        }
    }

    pub(crate) fn has_unknown_atoms(&self) -> bool {
        match self {
            Self::Feature(_) | Self::Opaque(_) => true,
            Self::Not(value) => value.has_unknown_atoms(),
            Self::All(values) | Self::Any(values) => values.iter().any(Self::has_unknown_atoms),
            Self::True | Self::False | Self::Test => false,
        }
    }

    pub(crate) fn canonical(&self) -> String {
        match self {
            Self::True => "true".into(),
            Self::False => "false".into(),
            Self::Test => "test".into(),
            Self::Feature(feature) => format!("feature={feature:?}"),
            Self::Opaque(value) => format!("opaque({value})"),
            Self::Not(value) => format!("not({})", value.canonical()),
            Self::All(values) => joined("all", values),
            Self::Any(values) => joined("any", values),
        }
    }

    fn opaque(meta: &Meta) -> Self {
        Self::Opaque(canonical_meta(meta))
    }
}

const fn truth(value: bool) -> CfgTruth {
    if value {
        CfgTruth::True
    } else {
        CfgTruth::False
    }
}

const fn invert(value: CfgTruth) -> CfgTruth {
    match value {
        CfgTruth::False => CfgTruth::True,
        CfgTruth::True => CfgTruth::False,
        CfgTruth::Unknown => CfgTruth::Unknown,
    }
}

const fn and(left: CfgTruth, right: CfgTruth) -> CfgTruth {
    match (left, right) {
        (CfgTruth::False, _) | (_, CfgTruth::False) => CfgTruth::False,
        (CfgTruth::True, CfgTruth::True) => CfgTruth::True,
        _ => CfgTruth::Unknown,
    }
}

const fn or(left: CfgTruth, right: CfgTruth) -> CfgTruth {
    match (left, right) {
        (CfgTruth::True, _) | (_, CfgTruth::True) => CfgTruth::True,
        (CfgTruth::False, CfgTruth::False) => CfgTruth::False,
        _ => CfgTruth::Unknown,
    }
}

#[cfg(test)]
#[path = "cfg_predicate_test.rs"]
mod cfg_predicate_test;
