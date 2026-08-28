//! Runtime-neutral async syntax remains distinct from runtime capability paths.

use syn::{ExprAsync, ExprAwait, visit};

use super::super::FactVisitor;

pub(in crate::source::visitor) fn visit_async(
    visitor: &mut FactVisitor<'_>,
    expression: &ExprAsync,
) {
    visitor.record_async_syntax(
        zrail_core::AsyncSyntax::AsyncBlock,
        expression.async_token.span,
    );
    visit::visit_expr_async(visitor, expression);
}

pub(in crate::source::visitor) fn visit_await(
    visitor: &mut FactVisitor<'_>,
    expression: &ExprAwait,
) {
    visitor.record_async_syntax(zrail_core::AsyncSyntax::Await, expression.await_token.span);
    visit::visit_expr_await(visitor, expression);
}
