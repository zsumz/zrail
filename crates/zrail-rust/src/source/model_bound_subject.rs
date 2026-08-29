//! Typed bound subjects preserve projections and qualified projection spelling.

use std::collections::BTreeSet;

use syn::{ExprPath, Type};

use super::super::{
    AssociatedSegment, GenericPathIdentity, ProjectionIdentity, fact::written_path,
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum BoundSubject {
    TypeParameter(String),
    SelfType,
    Projection {
        root: String,
        projection: ProjectionIdentity,
    },
}

impl BoundSubject {
    pub(crate) fn from_type(ty: &Type, declared: &BTreeSet<String>) -> Option<Self> {
        let Type::Path(path) = ty else {
            return None;
        };
        if let Some(qself) = &path.qself {
            let root = simple_root(&qself.ty, declared)?;
            let associated = path
                .path
                .segments
                .iter()
                .skip(qself.position)
                .map(AssociatedSegment::from_path)
                .collect::<Vec<_>>();
            if associated.is_empty() {
                return None;
            }
            let qualifying_trait = (qself.position > 0)
                .then(|| GenericPathIdentity::trait_path_prefix(&path.path, qself.position));
            return Some(Self::Projection {
                root,
                projection: ProjectionIdentity {
                    qualifying_trait,
                    associated,
                },
            });
        }
        if path.path.leading_colon.is_some() {
            return None;
        }
        let mut segments = path.path.segments.iter();
        let root = segments.next()?.ident.to_string();
        if !declared.iter().any(|name| visible(name) == visible(&root)) {
            return None;
        }
        let associated = segments
            .map(AssociatedSegment::from_path)
            .collect::<Vec<_>>();
        if associated.is_empty() {
            Some(if visible(&root) == "Self" {
                Self::SelfType
            } else {
                Self::TypeParameter(root)
            })
        } else {
            Some(Self::Projection {
                root,
                projection: ProjectionIdentity {
                    qualifying_trait: None,
                    associated,
                },
            })
        }
    }

    pub(crate) fn from_receiver(receiver: &str, declared: &BTreeSet<String>) -> Option<Self> {
        syn::parse_str::<Type>(receiver)
            .ok()
            .and_then(|ty| Self::from_type(&ty, declared))
    }

    pub(crate) fn from_expression(
        expression: &ExprPath,
        declared: &BTreeSet<String>,
    ) -> Option<(Self, String)> {
        let item = expression.path.segments.last()?.ident.to_string();
        if let Some(qself) = &expression.qself {
            let root = simple_root(&qself.ty, declared)?;
            let associated = expression
                .path
                .segments
                .iter()
                .skip(qself.position)
                .take(
                    expression
                        .path
                        .segments
                        .len()
                        .saturating_sub(qself.position + 1),
                )
                .map(AssociatedSegment::from_path)
                .collect::<Vec<_>>();
            if associated.is_empty() {
                return None;
            }
            let qualifying_trait = (qself.position > 0)
                .then(|| GenericPathIdentity::trait_path_prefix(&expression.path, qself.position));
            return Some((
                Self::Projection {
                    root,
                    projection: ProjectionIdentity {
                        qualifying_trait,
                        associated,
                    },
                },
                item,
            ));
        }
        if expression.path.leading_colon.is_some() || expression.path.segments.len() < 2 {
            return None;
        }
        let root = expression.path.segments.first()?.ident.to_string();
        if !declared.iter().any(|name| visible(name) == visible(&root)) {
            return None;
        }
        let associated = expression
            .path
            .segments
            .iter()
            .skip(1)
            .take(expression.path.segments.len().saturating_sub(2))
            .map(AssociatedSegment::from_path)
            .collect::<Vec<_>>();
        let subject = if associated.is_empty() {
            if visible(&root) == "Self" {
                Self::SelfType
            } else {
                Self::TypeParameter(root)
            }
        } else {
            Self::Projection {
                root,
                projection: ProjectionIdentity {
                    qualifying_trait: None,
                    associated,
                },
            }
        };
        Some((subject, item))
    }

    pub(crate) fn root(&self) -> &str {
        match self {
            Self::TypeParameter(root) | Self::Projection { root, .. } => root,
            Self::SelfType => "Self",
        }
    }

    pub(crate) fn projection(&self) -> Option<&ProjectionIdentity> {
        match self {
            Self::Projection { projection, .. } => Some(projection),
            Self::TypeParameter(_) | Self::SelfType => None,
        }
    }
}

fn simple_root(ty: &Type, declared: &BTreeSet<String>) -> Option<String> {
    let Type::Path(path) = ty else {
        return None;
    };
    if path.qself.is_some() || path.path.leading_colon.is_some() || path.path.segments.len() != 1 {
        return None;
    }
    let root = written_path(&path.path);
    declared
        .iter()
        .any(|name| visible(name) == visible(&root))
        .then_some(root)
}

fn visible(name: &str) -> &str {
    name.strip_prefix("r#").unwrap_or(name)
}

#[cfg(test)]
#[path = "model_bound_subject_test.rs"]
mod model_bound_subject_test;
