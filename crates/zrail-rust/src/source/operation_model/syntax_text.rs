//! Parseable source text retains generic arguments for deferred projection.

use syn::{Expr, GenericArgument, Path, PathArguments, ReturnType, Type, TypeParamBound};

pub(super) fn path_text(path: &Path) -> String {
    let mut text = segments_text(path, 0, path.segments.len());
    if path.leading_colon.is_some() {
        text.insert_str(0, "::");
    }
    text
}

pub(super) fn segments_text(path: &Path, start: usize, end: usize) -> String {
    let text = path
        .segments
        .iter()
        .skip(start)
        .take(end.saturating_sub(start))
        .map(segment_text)
        .collect::<Vec<_>>()
        .join("::");
    if text.is_empty() {
        "unresolved associated item".into()
    } else {
        text
    }
}

pub(super) fn type_text(ty: &Type) -> String {
    match ty {
        Type::Path(path) if path.qself.is_none() => path_text(&path.path),
        Type::Path(path) => qualified_type_text(path),
        Type::Reference(reference) => {
            let lifetime = reference
                .lifetime
                .as_ref()
                .map_or(String::new(), |value| format!("'{} ", value.ident));
            let mutable = reference.mutability.map_or("", |_| "mut ");
            format!("&{lifetime}{mutable}{}", type_text(&reference.elem))
        }
        Type::Tuple(tuple) => {
            let mut values = tuple.elems.iter().map(type_text).collect::<Vec<_>>();
            if values.len() == 1 {
                values[0].push(',');
            }
            format!("({})", values.join(", "))
        }
        Type::Slice(slice) => format!("[{}]", type_text(&slice.elem)),
        Type::Array(array) => format!("[{}; {}]", type_text(&array.elem), expr_text(&array.len)),
        Type::Ptr(pointer) => format!(
            "*{} {}",
            pointer.const_token.map_or("mut", |_| "const"),
            type_text(&pointer.elem)
        ),
        Type::Paren(paren) => format!("({})", type_text(&paren.elem)),
        Type::Group(group) => type_text(&group.elem),
        Type::Never(_) => "!".into(),
        Type::Infer(_) => "_".into(),
        _ => "unresolved_self_type".into(),
    }
}

fn qualified_type_text(path: &syn::TypePath) -> String {
    let Some(qself) = &path.qself else {
        return path_text(&path.path);
    };
    let qualifier = segments_text(&path.path, 0, qself.position);
    let associated = segments_text(&path.path, qself.position, path.path.segments.len());
    if qself.position == 0 {
        format!("<{}>::{associated}", type_text(&qself.ty))
    } else {
        format!("<{} as {qualifier}>::{associated}", type_text(&qself.ty))
    }
}

fn segment_text(segment: &syn::PathSegment) -> String {
    format!("{}{}", segment.ident, arguments_text(&segment.arguments))
}

fn arguments_text(arguments: &PathArguments) -> String {
    match arguments {
        PathArguments::None => String::new(),
        PathArguments::AngleBracketed(arguments) => {
            let prefix = arguments.colon2_token.map_or("", |_| "::");
            let values = arguments
                .args
                .iter()
                .map(argument_text)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{prefix}<{values}>")
        }
        PathArguments::Parenthesized(arguments) => {
            let inputs = arguments
                .inputs
                .iter()
                .map(type_text)
                .collect::<Vec<_>>()
                .join(", ");
            let output = match &arguments.output {
                ReturnType::Default => String::new(),
                ReturnType::Type(_, ty) => format!(" -> {}", type_text(ty)),
            };
            format!("({inputs}){output}")
        }
    }
}

fn argument_text(argument: &GenericArgument) -> String {
    match argument {
        GenericArgument::Lifetime(value) => format!("'{}", value.ident),
        GenericArgument::Type(value) => type_text(value),
        GenericArgument::Const(value) => expr_text(value),
        GenericArgument::AssocType(value) => format!(
            "{}{} = {}",
            value.ident,
            value.generics.as_ref().map_or(String::new(), angle_text),
            type_text(&value.ty)
        ),
        GenericArgument::AssocConst(value) => format!(
            "{}{} = {}",
            value.ident,
            value.generics.as_ref().map_or(String::new(), angle_text),
            expr_text(&value.value)
        ),
        GenericArgument::Constraint(value) => format!(
            "{}{}: {}",
            value.ident,
            value.generics.as_ref().map_or(String::new(), angle_text),
            value
                .bounds
                .iter()
                .map(bound_text)
                .collect::<Vec<_>>()
                .join(" + ")
        ),
        _ => "unresolved_argument".into(),
    }
}

fn angle_text(arguments: &syn::AngleBracketedGenericArguments) -> String {
    format!(
        "<{}>",
        arguments
            .args
            .iter()
            .map(argument_text)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn bound_text(bound: &TypeParamBound) -> String {
    match bound {
        TypeParamBound::Trait(bound) => path_text(&bound.path),
        TypeParamBound::Lifetime(value) => format!("'{}", value.ident),
        _ => "unresolved_bound".into(),
    }
}

fn expr_text(expression: &Expr) -> String {
    match expression {
        Expr::Path(path) if path.qself.is_none() => path_text(&path.path),
        Expr::Group(group) => expr_text(&group.expr),
        Expr::Paren(paren) => format!("({})", expr_text(&paren.expr)),
        Expr::Infer(_) => "_".into(),
        _ => "unresolved_const".into(),
    }
}
