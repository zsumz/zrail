//! Projection identity retains trait and associated-segment generic arguments.

#[path = "model_projection_identity/syntax.rs"]
mod syntax;

use syn::{Path, Type};
use zrail_core::AnalysisQuality;

use super::super::fact::written_path;
use syntax::{angle_arguments, arguments, path_arguments, peel_type};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum GenericArgumentsIdentity {
    Any,
    Exact(Vec<String>),
    Unknown,
}

impl GenericArgumentsIdentity {
    pub(crate) fn matches(&self, occurrence: &Self) -> bool {
        matches!(self, Self::Any | Self::Unknown)
            || matches!(occurrence, Self::Unknown)
            || self == occurrence
    }

    fn quality(&self) -> AnalysisQuality {
        if matches!(self, Self::Unknown) {
            AnalysisQuality::Unresolved
        } else {
            AnalysisQuality::Exact
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct GenericPathIdentity {
    pub(crate) path: String,
    pub(crate) arguments: GenericArgumentsIdentity,
}

impl GenericPathIdentity {
    pub(crate) fn trait_path(path: &Path) -> Self {
        Self {
            path: written_path(path),
            arguments: path_arguments(path, true),
        }
    }

    pub(crate) fn trait_path_prefix(path: &Path, segments: usize) -> Self {
        let path = Path {
            leading_colon: path.leading_colon,
            segments: path.segments.iter().take(segments).cloned().collect(),
        };
        Self::trait_path(&path)
    }

    pub(crate) fn type_path(ty: &Type) -> Option<Self> {
        let Type::Path(path) = peel_type(ty) else {
            return None;
        };
        path.qself.is_none().then(|| Self {
            path: written_path(&path.path),
            arguments: path_arguments(&path.path, false),
        })
    }

    pub(crate) fn wildcard(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            arguments: GenericArgumentsIdentity::Any,
        }
    }

    pub(crate) fn with_path(&self, path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            arguments: self.arguments.clone(),
        }
    }

    pub(crate) fn matches(&self, occurrence: &Self) -> bool {
        self.path == occurrence.path && self.arguments.matches(&occurrence.arguments)
    }

    pub(crate) fn quality(&self) -> AnalysisQuality {
        self.arguments.quality()
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct AssociatedSegment {
    pub(crate) name: String,
    pub(crate) arguments: GenericArgumentsIdentity,
}

impl AssociatedSegment {
    pub(crate) fn from_path(segment: &syn::PathSegment) -> Self {
        Self {
            name: segment.ident.to_string(),
            arguments: arguments(&segment.arguments, false),
        }
    }

    pub(crate) fn constrained(
        ident: &syn::Ident,
        generics: Option<&syn::AngleBracketedGenericArguments>,
    ) -> Self {
        Self {
            name: ident.to_string(),
            arguments: generics
                .map_or(GenericArgumentsIdentity::Exact(Vec::new()), angle_arguments),
        }
    }

    pub(crate) fn declaration(ident: &syn::Ident, has_generics: bool) -> Self {
        Self {
            name: ident.to_string(),
            arguments: if has_generics {
                GenericArgumentsIdentity::Any
            } else {
                GenericArgumentsIdentity::Exact(Vec::new())
            },
        }
    }

    pub(crate) fn matches(&self, occurrence: &Self) -> bool {
        self.name == occurrence.name && self.arguments.matches(&occurrence.arguments)
    }
}

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ProjectionIdentity {
    pub(crate) qualifying_trait: Option<GenericPathIdentity>,
    pub(crate) associated: Vec<AssociatedSegment>,
}

impl ProjectionIdentity {
    pub(crate) fn is_empty(&self) -> bool {
        self.associated.is_empty()
    }

    pub(crate) fn matches(&self, occurrence: &Self) -> bool {
        self.associated.len() == occurrence.associated.len()
            && self
                .associated
                .iter()
                .zip(&occurrence.associated)
                .all(|(left, right)| left.matches(right))
            && match (&self.qualifying_trait, &occurrence.qualifying_trait) {
                (Some(left), Some(right)) => left.matches(right),
                // Rust accepts an unqualified projection only when it can select one
                // associated type. Either the bound or the use may carry that explicit
                // qualifier, so the unqualified spelling remains compatible here.
                (None, _) | (_, None) => true,
            }
    }

    pub(crate) fn quality(&self) -> AnalysisQuality {
        self.qualifying_trait
            .as_ref()
            .map_or(AnalysisQuality::Exact, GenericPathIdentity::quality)
            .max(
                self.associated
                    .iter()
                    .fold(AnalysisQuality::Exact, |quality, segment| {
                        quality.max(segment.arguments.quality())
                    }),
            )
    }
}
