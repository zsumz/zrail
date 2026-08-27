//! Type-construction syntax records exact forms and conservative candidates.

use syn::{Expr, ExprCall, ExprPath, ExprStruct, spanned::Spanned};
use zrail_core::AnalysisQuality;

use super::{ConstructorForm, FactVisitor, SourceOperationKind, path_text};
use crate::source::{CfgPredicate, SyntaxGuard, operation_model::unwrapped};

impl FactVisitor<'_> {
    pub(in crate::source) fn record_struct_construction(&mut self, expression: &ExprStruct) {
        let identity = self.resolve_construction_identity(&expression.path);
        self.push_operation(
            SourceOperationKind::TypeConstruction,
            &identity,
            path_text(&expression.path),
            expression
                .path
                .segments
                .last()
                .map(|segment| segment.ident.span()),
            Some(ConstructorForm::Named),
        );
    }

    pub(in crate::source) fn record_call_construction(&mut self, call: &ExprCall) {
        let Expr::Path(callee) = unwrapped(call.func.as_ref()) else {
            return;
        };
        let mut identity = self.resolve_construction_identity(&callee.path);
        identity.quality = AnalysisQuality::Unresolved;
        let guard = self.constructor_candidate_guard(&callee.path);
        if guard.predicate().is_satisfiable() == Some(false) {
            return;
        }
        self.push_guarded_constructor_candidate(
            SourceOperationKind::TypeConstruction,
            &identity,
            path_text(&callee.path),
            callee
                .path
                .segments
                .last()
                .map(|segment| segment.ident.span()),
            ConstructorForm::Tuple,
            false,
            &guard,
        );
    }

    pub(in crate::source) fn record_path_construction(&mut self, expression: &ExprPath) {
        if self
            .constructor_path_exclusions
            .contains(&crate::source::fact::source_span(expression.path.span()))
        {
            return;
        }
        let mut identity = self.resolve_construction_identity(&expression.path);
        identity.quality = AnalysisQuality::Unresolved;
        let guard = self.constructor_candidate_guard(&expression.path);
        if guard.predicate().is_satisfiable() == Some(false) {
            return;
        }
        self.push_guarded_constructor_candidate(
            SourceOperationKind::ConstructorCapability,
            &identity,
            path_text(&expression.path),
            expression
                .path
                .segments
                .last()
                .map(|segment| segment.ident.span()),
            ConstructorForm::Unknown,
            false,
            &guard,
        );
    }

    fn constructor_candidate_guard(&self, path: &syn::Path) -> SyntaxGuard {
        let guard = self.syntax_guard();
        if path.leading_colon.is_some() || path.segments.len() != 1 {
            return guard;
        }
        let name = path.segments[0].ident.to_string();
        guard.combine(SyntaxGuard::from_predicate(CfgPredicate::not(
            self.local_value_shadow_guard(&name).predicate(),
        )))
    }
}
