//! Typed bound subjects preserve projections and qualified projection spelling.

use std::collections::BTreeSet;

use syn::Type;

use super::super::fact::written_path;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum BoundSubject {
    TypeParameter(String),
    SelfType,
    Projection {
        root: String,
        qualifying_trait: Option<String>,
        associated: Vec<String>,
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
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>();
            if associated.is_empty() {
                return None;
            }
            let qualifying_trait = (qself.position > 0).then(|| {
                let mut trait_path = path
                    .path
                    .segments
                    .iter()
                    .take(qself.position)
                    .map(|segment| segment.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::");
                if path.path.leading_colon.is_some() {
                    trait_path.insert_str(0, "::");
                }
                trait_path
            });
            return Some(Self::Projection {
                root,
                qualifying_trait,
                associated,
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
            .map(|segment| segment.ident.to_string())
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
                qualifying_trait: None,
                associated,
            })
        }
    }

    pub(crate) fn from_receiver(receiver: &str, declared: &BTreeSet<String>) -> Option<Self> {
        syn::parse_str::<Type>(receiver)
            .ok()
            .and_then(|ty| Self::from_type(&ty, declared))
    }

    pub(crate) fn root(&self) -> &str {
        match self {
            Self::TypeParameter(root) | Self::Projection { root, .. } => root,
            Self::SelfType => "Self",
        }
    }

    pub(crate) fn without_qualifier(&self) -> Self {
        match self {
            Self::Projection {
                root, associated, ..
            } => Self::Projection {
                root: root.clone(),
                qualifying_trait: None,
                associated: associated.clone(),
            },
            other => other.clone(),
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
