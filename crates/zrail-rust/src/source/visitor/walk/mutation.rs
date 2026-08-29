//! Mutation syntax routes through the shared place-expression model.

use syn::{
    BinOp, ExprAssign, ExprBinary, ExprRawAddr, ExprReference, PointerMutability,
    visit::{self, Visit},
};

use super::super::FactVisitor;

pub(in crate::source::visitor) fn visit_binary(
    visitor: &mut FactVisitor<'_>,
    expression: &ExprBinary,
) {
    if matches!(
        expression.op,
        BinOp::AddAssign(_)
            | BinOp::SubAssign(_)
            | BinOp::MulAssign(_)
            | BinOp::DivAssign(_)
            | BinOp::RemAssign(_)
            | BinOp::BitXorAssign(_)
            | BinOp::BitAndAssign(_)
            | BinOp::BitOrAssign(_)
            | BinOp::ShlAssign(_)
            | BinOp::ShrAssign(_)
    ) {
        visitor.with_place_operation(
            crate::source::SourceOperationKind::FieldWrite,
            &expression.left,
            |visitor| visit::visit_expr_binary(visitor, expression),
        );
    } else {
        visit::visit_expr_binary(visitor, expression);
    }
}

pub(in crate::source::visitor) fn visit_assign(
    visitor: &mut FactVisitor<'_>,
    expression: &ExprAssign,
) {
    for attribute in &expression.attrs {
        visitor.visit_attribute(attribute);
    }
    crate::source::assignee_expression::visit(visitor, &expression.left);
    visitor.visit_expr(&expression.right);
}

pub(in crate::source::visitor) fn visit_reference(
    visitor: &mut FactVisitor<'_>,
    expression: &ExprReference,
) {
    if expression.mutability.is_some() {
        visitor.with_place_operation(
            crate::source::SourceOperationKind::FieldMutableBorrow,
            &expression.expr,
            |visitor| visit::visit_expr_reference(visitor, expression),
        );
    } else {
        visit::visit_expr_reference(visitor, expression);
    }
}

pub(in crate::source::visitor) fn visit_raw_address(
    visitor: &mut FactVisitor<'_>,
    expression: &ExprRawAddr,
) {
    if matches!(expression.mutability, PointerMutability::Mut(_)) {
        visitor.with_place_operation(
            crate::source::SourceOperationKind::FieldMutableBorrow,
            &expression.expr,
            |visitor| visit::visit_expr_raw_addr(visitor, expression),
        );
    } else {
        visit::visit_expr_raw_addr(visitor, expression);
    }
}
