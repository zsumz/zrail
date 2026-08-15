//! Macro positions identify includes and unresolved item-producing expansion.

use syn::{ItemMacro, ItemMod, Macro, spanned::Spanned};
use zrail_core::AnalysisQuality;

use super::{
    attributes::is_cfg_test, fact::fact, includes::include_boundary, model::IncludeContext,
    visitor::FactVisitor,
};

impl FactVisitor<'_> {
    pub(super) fn record_module(&mut self, module: &ItemMod) {
        if let Some(unsafe_token) = &module.unsafety {
            self.unsafe_constructs.push(fact(
                "unsafe module",
                unsafe_token.span,
                AnalysisQuality::Exact,
            ));
        }
        let cfg_test = module.content.is_some() && module.attrs.iter().any(is_cfg_test);
        if cfg_test {
            self.tests.push(fact(
                format!("inline module {}", module.ident),
                module.ident.span(),
                AnalysisQuality::Exact,
            ));
        }
    }

    pub(super) fn record_item_macro(&mut self, item: &ItemMacro) {
        if let Some(mut boundary) = include_boundary(&item.mac, IncludeContext::Items) {
            boundary.cfg_test = self.test_only_context || item.attrs.iter().any(is_cfg_test);
            self.includes.push(boundary);
        } else if item.ident.is_none() {
            let (name, _) = self.imports.resolve(&item.mac.path);
            self.item_macros.push(fact(
                name,
                item.mac.path.span(),
                AnalysisQuality::Unresolved,
            ));
        }
    }

    pub(super) fn record_expression_macro(&mut self, invocation: &Macro, cfg_test: bool) {
        if let Some(mut boundary) = include_boundary(invocation, IncludeContext::Expression) {
            boundary.cfg_test = self.test_only_context || cfg_test;
            self.includes.push(boundary);
        }
    }
}
