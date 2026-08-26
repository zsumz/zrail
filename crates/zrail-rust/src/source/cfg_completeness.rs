//! Feature-dependent attributes that change target identity fail exact-world analysis.

use syn::visit::Visit;

use super::{attributes::feature_cfg_attr_requires_completeness, fact::source_span};

#[derive(Default)]
struct Collector {
    spans: Vec<zrail_core::SourceSpan>,
}

impl<'ast> Visit<'ast> for Collector {
    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        if feature_cfg_attr_requires_completeness(std::slice::from_ref(attribute)) {
            self.spans.push(source_span(attribute.pound_token.span));
        }
        syn::visit::visit_attribute(self, attribute);
    }
}

pub(in crate::source) fn file(syntax: &syn::File) -> Vec<zrail_core::SourceSpan> {
    let mut collector = Collector::default();
    collector.visit_file(syntax);
    collector.spans.sort();
    collector.spans.dedup();
    collector.spans
}

pub(in crate::source) fn expression(syntax: &syn::Expr) -> Vec<zrail_core::SourceSpan> {
    let mut collector = Collector::default();
    collector.visit_expr(syntax);
    collector.spans.sort();
    collector.spans.dedup();
    collector.spans
}

#[cfg(test)]
#[path = "cfg_completeness_test.rs"]
mod cfg_completeness_test;
