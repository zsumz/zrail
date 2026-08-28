//! Direct-call and test-function facts stay separate from traversal mechanics.

use syn::{ExprCall, ExprMethodCall, ItemFn};
use zrail_core::AnalysisQuality;

use super::{FactNamespace, FactVisitor, attributes::is_test_attribute, fact::fact};

impl FactVisitor<'_> {
    pub(in crate::source) fn record_call(&mut self, call: &ExprCall) {
        if let Some(path) = super::calls::callee_path(call.func.as_ref())
            && let Some(mut fact) = self.current_self_fact(&path.path, FactNamespace::Value)
        {
            fact.inherits_parent_context = self.inherits_parent_context;
            self.calls.push(fact);
            return;
        }
        let facts = super::calls::facts(
            call,
            self.imports,
            &self.syntax_guard(),
            &self.generic_types,
            &self.lexical_scope,
        );
        let scoped = self.with_implicit_prelude_scope(facts, true);
        let generic_root = scoped.iter().any(|fact| fact.generic_shadow.is_some());
        self.calls.extend(
            scoped
                .into_iter()
                .filter(|fact| !generic_root || fact.generic_shadow.is_some()),
        );
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
