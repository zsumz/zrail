//! Stable syntax keys distinguish generic arguments without token-string identity.

use syn::{Expr, GenericArgument, Path, PathArguments, ReturnType, Type};

use super::GenericArgumentsIdentity;
use crate::source::fact::written_path;

pub(super) fn path_arguments(path: &Path, trait_path: bool) -> GenericArgumentsIdentity {
    let mut keys = Vec::new();
    for segment in &path.segments {
        match arguments(&segment.arguments, trait_path) {
            GenericArgumentsIdentity::Exact(values) => keys.extend(values),
            GenericArgumentsIdentity::Unknown => return GenericArgumentsIdentity::Unknown,
            GenericArgumentsIdentity::Any => {}
        }
    }
    GenericArgumentsIdentity::Exact(keys)
}

pub(super) fn arguments(arguments: &PathArguments, trait_path: bool) -> GenericArgumentsIdentity {
    match arguments {
        PathArguments::None => GenericArgumentsIdentity::Exact(Vec::new()),
        PathArguments::AngleBracketed(values) => angle_arguments_filtered(values, trait_path),
        PathArguments::Parenthesized(values) => {
            let mut keys = values
                .inputs
                .iter()
                .map(type_key)
                .collect::<Option<Vec<_>>>();
            if let (Some(keys), ReturnType::Type(_, output)) = (&mut keys, &values.output) {
                keys.push(format!(
                    "output:{}",
                    type_key(output).unwrap_or_else(|| "?".into())
                ));
            }
            keys.map_or(
                GenericArgumentsIdentity::Unknown,
                GenericArgumentsIdentity::Exact,
            )
        }
    }
}

pub(super) fn angle_arguments(
    values: &syn::AngleBracketedGenericArguments,
) -> GenericArgumentsIdentity {
    angle_arguments_filtered(values, false)
}

fn angle_arguments_filtered(
    values: &syn::AngleBracketedGenericArguments,
    trait_path: bool,
) -> GenericArgumentsIdentity {
    let keys = values
        .args
        .iter()
        .filter(|argument| {
            !trait_path
                || !matches!(
                    argument,
                    GenericArgument::AssocType(_)
                        | GenericArgument::AssocConst(_)
                        | GenericArgument::Constraint(_)
                )
        })
        .map(argument_key)
        .collect::<Option<Vec<_>>>();
    keys.map_or(
        GenericArgumentsIdentity::Unknown,
        GenericArgumentsIdentity::Exact,
    )
}

fn argument_key(argument: &GenericArgument) -> Option<String> {
    match argument {
        GenericArgument::Lifetime(value) => Some(format!("lifetime:{value}")),
        GenericArgument::Type(value) => Some(format!("type:{}", type_key(value)?)),
        GenericArgument::Const(value) => Some(format!("const:{}", expression_key(value)?)),
        _ => None,
    }
}

fn type_key(ty: &Type) -> Option<String> {
    match peel_type(ty) {
        Type::Path(path) if path.qself.is_none() => Some(format!(
            "path:{}:{:?}",
            written_path(&path.path),
            path_arguments(&path.path, false)
        )),
        Type::Reference(value) => Some(format!(
            "ref:{}:{}",
            value.mutability.is_some(),
            type_key(&value.elem)?
        )),
        Type::Tuple(value) => Some(format!(
            "tuple:{:?}",
            value
                .elems
                .iter()
                .map(type_key)
                .collect::<Option<Vec<_>>>()?
        )),
        Type::Slice(value) => Some(format!("slice:{}", type_key(&value.elem)?)),
        Type::Array(value) => Some(format!(
            "array:{}:{}",
            type_key(&value.elem)?,
            expression_key(&value.len)?
        )),
        Type::Ptr(value) => Some(format!(
            "ptr:{}:{}",
            value.mutability.is_some(),
            type_key(&value.elem)?
        )),
        Type::Never(_) => Some("never".into()),
        Type::Infer(_) => Some("infer".into()),
        _ => None,
    }
}

fn expression_key(expression: &Expr) -> Option<String> {
    match expression {
        Expr::Path(value) if value.qself.is_none() => Some(written_path(&value.path)),
        Expr::Lit(value) => Some(format!("{:?}", value.lit)),
        Expr::Group(value) => expression_key(&value.expr),
        Expr::Paren(value) => expression_key(&value.expr),
        _ => None,
    }
}

pub(super) fn peel_type(ty: &Type) -> &Type {
    match ty {
        Type::Group(value) => peel_type(&value.elem),
        Type::Paren(value) => peel_type(&value.elem),
        _ => ty,
    }
}
