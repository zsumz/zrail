//! Destructuring assignees expose written places without pretending to construct values.

use syn::{Expr, ExprCall, ExprStruct, visit::Visit};

use super::{SourceOperationKind, visitor_parts::FactVisitor};

pub(super) fn visit(visitor: &mut FactVisitor<'_>, expression: &Expr) {
    match expression {
        Expr::Array(array) => visitor.with_cfg(&array.attrs, |visitor| {
            visit_attributes(visitor, &array.attrs);
            for element in &array.elems {
                visit(visitor, element);
            }
        }),
        Expr::Call(call) => visit_tuple_struct(visitor, call),
        Expr::Group(group) => visitor.with_cfg(&group.attrs, |visitor| {
            visit_attributes(visitor, &group.attrs);
            visit(visitor, &group.expr);
        }),
        Expr::Paren(paren) => visitor.with_cfg(&paren.attrs, |visitor| {
            visit_attributes(visitor, &paren.attrs);
            visit(visitor, &paren.expr);
        }),
        Expr::Path(path) => visitor.with_cfg(&path.attrs, |visitor| {
            visitor.record_expression_path(path);
        }),
        Expr::Struct(structure) => visit_struct(visitor, structure),
        Expr::Tuple(tuple) => visitor.with_cfg(&tuple.attrs, |visitor| {
            visit_attributes(visitor, &tuple.attrs);
            for element in &tuple.elems {
                visit(visitor, element);
            }
        }),
        Expr::Infer(_) | Expr::Range(_) => visitor.visit_expr(expression),
        _ => visitor.with_place_operation(SourceOperationKind::FieldWrite, expression, |visitor| {
            visitor.visit_expr(expression);
        }),
    }
}

fn visit_struct(visitor: &mut FactVisitor<'_>, structure: &ExprStruct) {
    visitor.with_cfg(&structure.attrs, |visitor| {
        visit_attributes(visitor, &structure.attrs);
        visit_type_path(visitor, structure.qself.as_ref(), &structure.path);
        for field in &structure.fields {
            visitor.with_cfg(&field.attrs, |visitor| {
                visit_attributes(visitor, &field.attrs);
                if !ignored(&field.expr) {
                    visitor.record_assignee_source_field(&structure.path, &field.member);
                }
                visit(visitor, &field.expr);
            });
        }
    });
}

fn visit_tuple_struct(visitor: &mut FactVisitor<'_>, call: &ExprCall) {
    visitor.with_cfg(&call.attrs, |visitor| {
        visit_attributes(visitor, &call.attrs);
        if let Expr::Path(path) = call.func.as_ref() {
            visit_type_path(visitor, path.qself.as_ref(), &path.path);
        } else {
            visitor.visit_expr(&call.func);
        }
        for argument in &call.args {
            visit(visitor, argument);
        }
    });
}

fn visit_type_path(visitor: &mut FactVisitor<'_>, qself: Option<&syn::QSelf>, path: &syn::Path) {
    if let Some(qself) = qself {
        visitor.visit_type(&qself.ty);
    }
    visitor.with_pattern_type_paths(|visitor| visitor.visit_path(path));
}

fn visit_attributes(visitor: &mut FactVisitor<'_>, attributes: &[syn::Attribute]) {
    for attribute in attributes {
        visitor.visit_attribute(attribute);
    }
}

fn ignored(expression: &Expr) -> bool {
    match expression {
        Expr::Group(group) => ignored(&group.expr),
        Expr::Infer(_) => true,
        Expr::Paren(paren) => ignored(&paren.expr),
        _ => false,
    }
}
