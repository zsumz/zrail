//! Generic roots shadow only the Rust namespace that declares them.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RootLookupNamespace {
    Type,
    Value,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GenericRootShadow {
    TypeParameter,
    ConstParameter,
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
    if written.starts_with("::") {
        return None;
    }
    written
        .split("::")
        .next()
        .filter(|root| !root.is_empty())
        .filter(|root| !matches!(*root, "crate" | "self" | "super" | "Self"))
}

fn contains(generics: &[String], root: &str) -> bool {
    let root = root.strip_prefix("r#").unwrap_or(root);
    generics
        .iter()
        .any(|generic| generic.strip_prefix("r#").unwrap_or(generic) == root)
}
