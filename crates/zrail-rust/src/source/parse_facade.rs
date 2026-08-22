//! Facade and entrypoint facts distinguish declarations from implementation.

use syn::{Item, spanned::Spanned};
use zrail_core::AnalysisQuality;

use super::{ObservedFact, fact::fact};

pub(super) fn items(relative: &str, syntax: &syn::File) -> Vec<ObservedFact> {
    syntax
        .items
        .iter()
        .filter_map(|item| {
            if declarative(relative, item) {
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

fn declarative(relative: &str, item: &Item) -> bool {
    match item {
        Item::Mod(module) if module.content.is_none() => true,
        Item::Use(_) | Item::ExternCrate(_) => true,
        Item::Fn(function) => relative.ends_with("/main.rs") && function.sig.ident == "main",
        _ => false,
    }
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
        _ => "item".into(),
    }
}
