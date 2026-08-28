//! Generic roots shadow only the Rust namespace that declares them.

use zrail_core::AnalysisQuality;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RootLookupNamespace {
    Type,
    Value,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum GenericRootShadow {
    TypeParameter,
    ConstParameter,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GenericRootIdentity {
    pub(crate) name: String,
    pub(crate) shadow: GenericRootShadow,
    pub(crate) quality: AnalysisQuality,
}

impl GenericRootIdentity {
    pub(crate) fn is_associated(&self) -> bool {
        self.quality == AnalysisQuality::Unresolved
    }
}

pub(crate) fn generic_root_identity(
    written: &str,
    lookup: RootLookupNamespace,
    generic_types: &[String],
    generic_values: &[String],
) -> Option<GenericRootIdentity> {
    let shadow = generic_root_shadow(written, lookup, generic_types, generic_values)?;
    Some(identity_for_generic_root(written, shadow))
}

pub(crate) fn identity_for_generic_root(
    written: &str,
    shadow: GenericRootShadow,
) -> GenericRootIdentity {
    let root = written_root(written).unwrap_or(written);
    let visible = root.strip_prefix("r#").unwrap_or(root);
    let label = match shadow {
        GenericRootShadow::TypeParameter => "type-parameter",
        GenericRootShadow::ConstParameter => "const-parameter",
    };
    let suffix = &written[root.len()..];
    GenericRootIdentity {
        name: format!("<{label} {visible}>{suffix}"),
        shadow,
        quality: if suffix.is_empty() {
            AnalysisQuality::Exact
        } else {
            AnalysisQuality::Unresolved
        },
    }
}

pub(crate) fn generic_root_shadow(
    written: &str,
    lookup: RootLookupNamespace,
    generic_types: &[String],
    generic_values: &[String],
) -> Option<GenericRootShadow> {
    let root = generic_root(written)?;
    match lookup {
        RootLookupNamespace::Type => {
            contains(generic_types, root).then_some(GenericRootShadow::TypeParameter)
        }
        RootLookupNamespace::Value => {
            contains(generic_values, root).then_some(GenericRootShadow::ConstParameter)
        }
    }
}

fn generic_root(written: &str) -> Option<&str> {
    written_root(written).filter(|root| !matches!(*root, "crate" | "self" | "super" | "Self"))
}

fn written_root(written: &str) -> Option<&str> {
    (!written.starts_with("::"))
        .then(|| written.split("::").next())
        .flatten()
        .filter(|root| !root.is_empty())
}

fn contains(generics: &[String], root: &str) -> bool {
    let root = root.strip_prefix("r#").unwrap_or(root);
    generics
        .iter()
        .any(|generic| generic.strip_prefix("r#").unwrap_or(generic) == root)
}
