//! Macro positions identify includes and unresolved item-producing expansion.

use syn::{ItemMacro, Macro, spanned::Spanned};
use zrail_core::AnalysisQuality;

use super::{fact::fact, includes::include_boundary, model::IncludeContext, visitor::FactVisitor};

impl FactVisitor<'_> {
    pub(super) fn record_item_macro(&mut self, item: &ItemMacro) {
        if let Some(boundary) = include_boundary(&item.mac, IncludeContext::Items) {
            self.includes.push(boundary);
        } else if item.ident.is_none() {
            self.item_macros.push(fact(
                macro_name(&item.mac),
                item.mac.path.span(),
                AnalysisQuality::Unresolved,
            ));
        }
    }

    pub(super) fn record_expression_macro(&mut self, invocation: &Macro) {
        if let Some(boundary) = include_boundary(invocation, IncludeContext::Expression) {
            self.includes.push(boundary);
        }
    }
}

fn macro_name(invocation: &Macro) -> String {
    invocation
        .path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}
