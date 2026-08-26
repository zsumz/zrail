//! Stable cfg predicate text and algebra helpers stay outside the core model.

use syn::{Expr, ExprLit, Lit, Meta, Token, punctuated::Punctuated};

use super::CfgPredicate;

pub(super) fn canonical_meta(meta: &Meta) -> String {
    match meta {
        Meta::Path(path) => canonical_path(path),
        Meta::NameValue(value) => format!(
            "{}={}",
            canonical_path(&value.path),
            canonical_expr(&value.value)
        ),
        Meta::List(list) => arguments(list).map_or_else(
            || format!("{}({})", canonical_path(&list.path), list.tokens),
            |values| {
                let values = values
                    .iter()
                    .map(canonical_meta)
                    .collect::<Vec<_>>()
                    .join(",");
                format!("{}({values})", canonical_path(&list.path))
            },
        ),
    }
}

pub(super) fn arguments(list: &syn::MetaList) -> Option<Punctuated<Meta, Token![,]>> {
    list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        .ok()
}

pub(super) fn has_inverse(values: &[CfgPredicate]) -> bool {
    values.iter().any(|value| match value {
        CfgPredicate::Not(nested) => values.binary_search(nested.as_ref()).is_ok(),
        value => values
            .binary_search(&CfgPredicate::Not(Box::new(value.clone())))
            .is_ok(),
    })
}

pub(super) fn joined(kind: &str, values: &[CfgPredicate]) -> String {
    let values = values
        .iter()
        .map(CfgPredicate::canonical)
        .collect::<Vec<_>>()
        .join(",");
    format!("{kind}({values})")
}

fn canonical_path(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn canonical_expr(expression: &Expr) -> String {
    let Expr::Lit(ExprLit { lit, .. }) = expression else {
        return "<unsupported-expression>".into();
    };
    match lit {
        Lit::Str(value) => format!("{:?}", value.value()),
        Lit::ByteStr(value) => format!("{:?}", value.value()),
        Lit::Byte(value) => value.value().to_string(),
        Lit::Char(value) => format!("{:?}", value.value()),
        Lit::Int(value) => value.base10_digits().into(),
        Lit::Float(value) => value.base10_digits().into(),
        Lit::Bool(value) => value.value.to_string(),
        Lit::Verbatim(value) => value.to_string(),
        _ => "<unsupported-literal>".into(),
    }
}
