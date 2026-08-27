//! Operation-bearing expressions preserve extraction order during traversal.

use syn::{ExprCall, ExprField, ExprMethodCall, ExprStruct, spanned::Spanned, visit};

use super::FactVisitor;

pub(super) fn visit_call(visitor: &mut FactVisitor<'_>, call: &ExprCall) {
    let checkpoint = visitor.constructor_path_exclusions.len();
    if let syn::Expr::Path(callee) = super::super::operation_model::unwrapped(call.func.as_ref()) {
        visitor
            .constructor_path_exclusions
            .push(super::super::fact::source_span(callee.path.span()));
    }
    visitor.record_call_construction(call);
    visitor.record_call(call);
    visit::visit_expr_call(visitor, call);
    visitor.constructor_path_exclusions.truncate(checkpoint);
}

pub(super) fn visit_method_call(visitor: &mut FactVisitor<'_>, call: &ExprMethodCall) {
    visitor.record_field_receiver_call(call);
    visitor.record_method_operation(call);
    visitor.record_method_call(call);
    visit::visit_expr_method_call(visitor, call);
}

pub(super) fn visit_struct(visitor: &mut FactVisitor<'_>, expression: &ExprStruct) {
    visitor.record_struct_construction(expression);
    visitor.record_struct_update_reads(expression);
    visit::visit_expr_struct(visitor, expression);
}

pub(super) fn visit_field(visitor: &mut FactVisitor<'_>, expression: &ExprField) {
    visitor.record_field_read(expression);
    visit::visit_expr_field(visitor, expression);
}
