//! Facade and entrypoint facts distinguish declarations from behavior.

use syn::{Expr, Item, ItemFn, Stmt, spanned::Spanned};
use zrail_core::AnalysisQuality;

use crate::inventory::FileClass;

use super::super::{ObservedFact, fact::fact};

pub(super) fn items(class: FileClass, syntax: &syn::File) -> Vec<ObservedFact> {
    syntax
        .items
        .iter()
        .filter_map(|item| {
            if declarative(class, item) {
                None
            } else {
                let span = match item {
                    Item::Macro(item_macro) => item_macro.mac.span(),
                    _ => item.span(),
                };
                Some(fact(kind(item), span, AnalysisQuality::Exact))
            }
        })
        .collect()
}

fn declarative(class: FileClass, item: &Item) -> bool {
    match item {
        Item::Mod(module) if module.content.is_none() => true,
        Item::Use(_)
        | Item::ExternCrate(_)
        | Item::Const(_)
        | Item::Enum(_)
        | Item::Struct(_)
        | Item::Type(_)
        | Item::Union(_) => true,
        Item::Fn(function) if entrypoint(class, function) => thin(function),
        _ => false,
    }
}

fn entrypoint(class: FileClass, function: &ItemFn) -> bool {
    (class == FileClass::EntryPoint && function.sig.ident == "main")
        || (class == FileClass::Facade && proc_macro_entrypoint(function))
}

fn proc_macro_entrypoint(function: &ItemFn) -> bool {
    function.attrs.iter().any(|attribute| {
        ["proc_macro", "proc_macro_attribute", "proc_macro_derive"]
            .iter()
            .any(|name| attribute.path().is_ident(name))
    })
}

fn thin(function: &ItemFn) -> bool {
    match function.block.stmts.as_slice() {
        [] => true,
        [Stmt::Expr(expression, _)] => handoff(expression),
        _ => false,
    }
}

fn handoff(expression: &Expr) -> bool {
    match expression {
        Expr::Await(value) => handoff(&value.base),
        Expr::Call(value) => safe_expression(&value.func) && handoff_inputs(&value.args),
        Expr::Cast(value) => handoff(&value.expr),
        Expr::Group(value) => handoff(&value.expr),
        Expr::MethodCall(value) => {
            handoff_inputs(std::iter::once(value.receiver.as_ref()).chain(value.args.iter()))
        }
        Expr::Paren(value) => handoff(&value.expr),
        Expr::Return(value) => value.expr.as_deref().is_some_and(handoff),
        Expr::Try(value) => handoff(&value.expr),
        _ => false,
    }
}

fn safe_expression(expression: &Expr) -> bool {
    match expression {
        Expr::Array(value) => safe_expressions(&value.elems),
        Expr::Await(value) => safe_expression(&value.base),
        Expr::Cast(value) => safe_expression(&value.expr),
        Expr::Field(value) => safe_expression(&value.base),
        Expr::Group(value) => safe_expression(&value.expr),
        Expr::Index(value) => safe_expression(&value.expr) && safe_expression(&value.index),
        Expr::Lit(_) | Expr::Path(_) => true,
        Expr::Paren(value) => safe_expression(&value.expr),
        Expr::Reference(value) => safe_expression(&value.expr),
        Expr::Struct(value) => {
            let fields = safe_expressions(value.fields.iter().map(|field| &field.expr));
            let rest = match value.rest.as_deref() {
                Some(expression) => safe_expression(expression),
                None => true,
            };
            fields && rest
        }
        Expr::Try(value) => safe_expression(&value.expr),
        Expr::Tuple(value) => safe_expressions(&value.elems),
        _ => false,
    }
}

fn safe_expressions<'a>(expressions: impl IntoIterator<Item = &'a Expr>) -> bool {
    expressions.into_iter().all(safe_expression)
}

fn handoff_inputs<'a>(expressions: impl IntoIterator<Item = &'a Expr>) -> bool {
    let mut nested_handoffs = 0;
    expressions.into_iter().all(|expression| {
        if safe_expression(expression) {
            return true;
        }
        if !handoff(expression) {
            return false;
        }
        nested_handoffs += 1;
        nested_handoffs == 1
    })
}

fn kind(item: &Item) -> String {
    match item {
        Item::Const(_) => "const".into(),
        Item::Enum(_) => "enum".into(),
        Item::Fn(_) => "function".into(),
        Item::Impl(_) => "impl".into(),
        Item::Macro(item_macro) => item_macro
            .mac
            .path
            .segments
            .last()
            .map_or_else(|| "macro".into(), |segment| format!("{}!", segment.ident)),
        Item::Static(_) => "static".into(),
        Item::Struct(_) => "struct".into(),
        Item::Trait(_) => "trait".into(),
        Item::Type(_) => "type".into(),
        Item::Union(_) => "union".into(),
        Item::Mod(_) => "inline module".into(),
        _ => "item".into(),
    }
}

#[cfg(test)]
#[path = "parse_facade_test.rs"]
mod parse_facade_test;
