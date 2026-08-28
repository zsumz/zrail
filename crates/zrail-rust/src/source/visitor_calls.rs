//! Direct-call and test-function facts stay separate from traversal mechanics.

use syn::{ExprCall, ExprMethodCall, ItemFn};
use zrail_core::AnalysisQuality;

use super::{FactVisitor, attributes::is_test_attribute, fact::fact};

impl FactVisitor<'_> {
    pub(in crate::source) fn record_call(&mut self, call: &ExprCall) {
        let facts = super::calls::facts(
            call,
            self.imports,
            &self.syntax_guard(),
            &self.generic_types,
            &self.lexical_scope,
        );
        self.calls
            .extend(self.with_implicit_prelude_scope(facts, true));
    }

    pub(in crate::source) fn record_method_call(&mut self, call: &ExprMethodCall) {
        self.methods.push(fact(
            call.method.to_string(),
            call.method.span(),
            AnalysisQuality::Conservative,
        ));
    }

    pub(in crate::source) fn record_test_function(&mut self, function: &ItemFn) {
        if function.attrs.iter().any(is_test_attribute) {
            self.tests.push(fact(
                function.sig.ident.to_string(),
                function.sig.ident.span(),
                AnalysisQuality::Exact,
            ));
        }
    }
}
