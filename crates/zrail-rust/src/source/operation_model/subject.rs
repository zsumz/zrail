//! One qself model is shared by call and construction extraction.

use std::borrow::Cow;

use syn::{ExprPath, Path, Type};

use super::syntax_text::{segments_text, type_text as syntax_type_text};
use crate::source::{RootLookupNamespace, fact::written_path};

#[derive(Clone, Copy)]
pub(in crate::source) enum WrittenOperationSubject<'a> {
    Path(&'a Path),
    TypeRelative {
        self_type: &'a Type,
        path: &'a Path,
    },
    TraitQualified {
        self_type: &'a Type,
        path: &'a Path,
        trait_segments: usize,
    },
}

fn type_text(ty: &Type) -> String {
    match ty {
        Type::Path(path) if path.qself.is_none() => syntax_type_text(ty),
        Type::Reference(reference) => format!("&{}", type_text(&reference.elem)),
        _ => "unresolved self type".into(),
    }
}

impl<'a> WrittenOperationSubject<'a> {
    pub(in crate::source) fn from_expression(expression: &'a ExprPath) -> Self {
        match &expression.qself {
            None => Self::Path(&expression.path),
            Some(qself) if qself.position == 0 => Self::TypeRelative {
                self_type: &qself.ty,
                path: &expression.path,
            },
            Some(qself) => Self::TraitQualified {
                self_type: &qself.ty,
                path: &expression.path,
                trait_segments: qself.position,
            },
        }
    }

    pub(in crate::source) fn call_path(self) -> Option<Cow<'a, Path>> {
        match self {
            Self::Path(path) | Self::TraitQualified { path, .. } => Some(Cow::Borrowed(path)),
            Self::TypeRelative { self_type, path } => {
                Some(Cow::Owned(join_self(self_type, path.segments.iter())?))
            }
        }
    }

    pub(in crate::source) fn construction_path(self) -> Option<Cow<'a, Path>> {
        match self {
            Self::Path(path) => Some(Cow::Borrowed(path)),
            Self::TypeRelative { self_type, path } => {
                Some(Cow::Owned(join_self(self_type, path.segments.iter())?))
            }
            Self::TraitQualified {
                self_type,
                path,
                trait_segments,
            } => Some(Cow::Owned(join_self(
                self_type,
                path.segments.iter().skip(trait_segments),
            )?)),
        }
    }

    pub(in crate::source) fn explicit_trait_path(self) -> Option<Path> {
        let Self::TraitQualified {
            path,
            trait_segments,
            ..
        } = self
        else {
            return None;
        };
        (self.associated_segments() == Some(1)).then(|| Path {
            leading_colon: path.leading_colon,
            segments: path.segments.iter().take(trait_segments).cloned().collect(),
        })
    }

    pub(in crate::source) fn written(self) -> String {
        match self {
            Self::Path(path) => written_path(path),
            Self::TypeRelative { self_type, path } => {
                format!(
                    "<{}>::{}",
                    type_text(self_type),
                    segments_text(path, 0, path.segments.len())
                )
            }
            Self::TraitQualified {
                self_type,
                path,
                trait_segments,
            } => {
                let mut trait_path = segments_text(path, 0, trait_segments);
                if path.leading_colon.is_some() {
                    trait_path.insert_str(0, "::");
                }
                let associated = segments_text(path, trait_segments, path.segments.len());
                format!("<{} as {trait_path}>::{associated}", type_text(self_type))
            }
        }
    }

    pub(in crate::source) const fn is_qualified(self) -> bool {
        !matches!(self, Self::Path(_))
    }

    pub(in crate::source) const fn is_trait_qualified(self) -> bool {
        matches!(self, Self::TraitQualified { .. })
    }

    pub(in crate::source) fn root_lookup(self) -> RootLookupNamespace {
        match self {
            Self::Path(path) if path.leading_colon.is_none() && path.segments.len() == 1 => {
                RootLookupNamespace::Value
            }
            Self::Path(_) | Self::TypeRelative { .. } | Self::TraitQualified { .. } => {
                RootLookupNamespace::Type
            }
        }
    }

    pub(in crate::source) fn associated_segments(self) -> Option<usize> {
        match self {
            Self::Path(_) => None,
            Self::TypeRelative { path, .. } => Some(path.segments.len()),
            Self::TraitQualified {
                path,
                trait_segments,
                ..
            } => path.segments.len().checked_sub(trait_segments),
        }
    }

    pub(in crate::source) fn force_unresolved(self, generic_types: &[String]) -> bool {
        if !self.is_qualified() || self.associated_segments() != Some(1) {
            return self.is_qualified() && self.associated_segments() != Some(1);
        }
        let Some(path) = self.self_path() else {
            return true;
        };
        let Some(root) = path.path.segments.first() else {
            return true;
        };
        root.ident != "Self"
            && generic_types
                .iter()
                .any(|generic| generic == &root.ident.to_string())
    }

    fn self_path(self) -> Option<&'a syn::TypePath> {
        let self_type = match self {
            Self::Path(_) => return None,
            Self::TypeRelative { self_type, .. } | Self::TraitQualified { self_type, .. } => {
                self_type
            }
        };
        match self_type {
            Type::Path(path) if path.qself.is_none() => Some(path),
            _ => None,
        }
    }
}

fn join_self<'a>(
    self_type: &Type,
    suffix: impl Iterator<Item = &'a syn::PathSegment>,
) -> Option<Path> {
    let Type::Path(self_path) = self_type else {
        return None;
    };
    if self_path.qself.is_some() {
        return None;
    }
    let mut path = self_path.path.clone();
    path.segments.extend(suffix.cloned());
    Some(path)
}
