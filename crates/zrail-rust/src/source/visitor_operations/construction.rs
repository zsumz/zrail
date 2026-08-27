//! Type-construction syntax records exact forms and conservative candidates.

use syn::{Expr, ExprCall, ExprPath, ExprStruct};
use zrail_core::AnalysisQuality;

use super::{
    ConstructorForm, FactVisitor, SourceOperationKind, last_segment_looks_constructor, path_text,
};

impl FactVisitor<'_> {
    pub(in crate::source) fn record_struct_construction(&mut self, expression: &ExprStruct) {
        let identity = self.resolve_identity(&expression.path);
        self.push_operation(
            SourceOperationKind::TypeConstruction,
            &identity,
            path_text(&expression.path),
            expression
                .path
                .segments
                .last()
                .map(|segment| segment.ident.span()),
            true,
        );
    }

    pub(in crate::source) fn record_call_construction(&mut self, call: &ExprCall) {
        let Expr::Path(callee) = call.func.as_ref() else {
            return;
        };
        let Some((form, proven)) = self.constructor_form(&callee.path) else {
            return;
        };
        if form != ConstructorForm::Tuple {
            return;
        }
        let mut identity = self.resolve_identity(&callee.path);
        if !proven {
            identity.quality = AnalysisQuality::Unresolved;
        }
        self.push_operation(
            SourceOperationKind::TypeConstruction,
            &identity,
            path_text(&callee.path),
            callee
                .path
                .segments
                .last()
                .map(|segment| segment.ident.span()),
            proven,
        );
    }

    pub(in crate::source) fn record_path_construction(&mut self, expression: &ExprPath) {
        let exact = self.constructor_form(&expression.path) == Some((ConstructorForm::Unit, true));
        if !exact && !last_segment_looks_constructor(&expression.path) {
            return;
        }
        let mut identity = self.resolve_identity(&expression.path);
        if !exact {
            identity.quality = AnalysisQuality::Unresolved;
        }
        self.push_operation(
            SourceOperationKind::TypeConstruction,
            &identity,
            path_text(&expression.path),
            expression
                .path
                .segments
                .last()
                .map(|segment| segment.ident.span()),
            exact,
        );
    }
}
