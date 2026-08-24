//! Direct-call and test-function facts stay separate from traversal mechanics.

use syn::{ExprCall, ExprMethodCall, ItemFn};
use zrail_core::AnalysisQuality;

use super::{attributes::is_test_attribute, fact::fact, visitor::FactVisitor};

impl FactVisitor<'_> {
    pub(super) fn record_call(&mut self, call: &ExprCall) {
        if let Some(unresolved) = super::calls::unresolved_projection(call, self.syntax_guard()) {
            self.call_resolutions.push(unresolved);
            return;
        }
        self.calls.extend(super::calls::facts(
            call,
            self.imports,
            self.syntax_guard(),
            &self.generic_types,
            &self.lexical_scope,
        ));
    }

    pub(super) fn record_method_call(&mut self, call: &ExprMethodCall) {
        self.methods.push(fact(
            call.method.to_string(),
            call.method.span(),
            AnalysisQuality::Conservative,
        ));
    }

    pub(super) fn record_test_function(&mut self, function: &ItemFn) {
        if function.attrs.iter().any(is_test_attribute) {
            self.tests.push(fact(
                function.sig.ident.to_string(),
                function.sig.ident.span(),
                AnalysisQuality::Exact,
            ));
        }
    }
}
