//! Conditional bindings follow Rust's left-to-right scope and shadowing rules.

use syn::{Arm, BinOp, Expr, ExprIf, ExprMatch, ExprWhile, visit::Visit};

use super::FactVisitor;

pub(super) fn visit_match(visitor: &mut FactVisitor<'_>, expression: &ExprMatch) {
    visit_attributes(visitor, &expression.attrs);
    visitor.visit_expr(&expression.expr);
    let input = visitor.pattern_input_from_expr(&expression.expr);
    for arm in &expression.arms {
        visitor.with_pattern_input(input, |visitor| visitor.visit_arm(arm));
    }
}

pub(super) fn visit_arm(visitor: &mut FactVisitor<'_>, arm: &Arm) {
    let input = visitor.current_pattern_input();
    visitor.visit_pat(&arm.pat);
    visitor.with_pattern_values(&arm.pat, input, |visitor| {
        let checkpoint = visitor.value_scope_checkpoint();
        if let Some((_, guard)) = &arm.guard {
            visit_condition(visitor, guard);
        }
        visitor.visit_expr(&arm.body);
        visitor.restore_value_scopes(checkpoint);
    });
}

pub(super) fn visit_if(visitor: &mut FactVisitor<'_>, expression: &ExprIf) {
    visit_attributes(visitor, &expression.attrs);
    let checkpoint = visitor.value_scope_checkpoint();
    visit_condition(visitor, &expression.cond);
    visitor.visit_block(&expression.then_branch);
    visitor.restore_value_scopes(checkpoint);
    if let Some((_, otherwise)) = &expression.else_branch {
        visitor.visit_expr(otherwise);
    }
}

pub(super) fn visit_while(visitor: &mut FactVisitor<'_>, expression: &ExprWhile) {
    visit_attributes(visitor, &expression.attrs);
    let checkpoint = visitor.value_scope_checkpoint();
    visit_condition(visitor, &expression.cond);
    visitor.visit_block(&expression.body);
    visitor.restore_value_scopes(checkpoint);
}

fn visit_condition(visitor: &mut FactVisitor<'_>, expression: &Expr) {
    match expression {
        Expr::Binary(binary) if matches!(binary.op, BinOp::And(_)) => {
            visitor.with_cfg(&binary.attrs, |visitor| {
                visit_attributes(visitor, &binary.attrs);
                visit_condition(visitor, &binary.left);
                visit_condition(visitor, &binary.right);
            });
        }
        Expr::Let(binding) => {
            visitor.with_cfg(&binding.attrs, |visitor| {
                visit_attributes(visitor, &binding.attrs);
                visitor.visit_expr(&binding.expr);
                let input = visitor.pattern_input_from_expr(&binding.expr);
                visitor.with_pattern_input(input, |visitor| visitor.visit_pat(&binding.pat));
                visitor.push_pattern_values(&binding.pat, input);
            });
        }
        Expr::Group(group) => {
            visitor.with_cfg(&group.attrs, |visitor| {
                visit_attributes(visitor, &group.attrs);
                visit_condition(visitor, &group.expr);
            });
        }
        Expr::Paren(paren) => {
            visitor.with_cfg(&paren.attrs, |visitor| {
                visit_attributes(visitor, &paren.attrs);
                visit_condition(visitor, &paren.expr);
            });
        }
        _ => visitor.visit_expr(expression),
    }
}

fn visit_attributes(visitor: &mut FactVisitor<'_>, attributes: &[syn::Attribute]) {
    for attribute in attributes {
        visitor.visit_attribute(attribute);
    }
}
