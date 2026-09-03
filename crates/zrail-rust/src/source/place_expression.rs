//! Structural Rust place projections keep mutation authority separate from value reads.

use syn::{Expr, ExprField, Member, UnOp};
use zrail_core::SourceSpan;

use super::fact::source_span;

pub(super) struct PlaceExpression<'a> {
    authority_fields: Vec<&'a ExprField>,
    direct_field: Option<&'a ExprField>,
    excluded_reads: Vec<SourceSpan>,
}

impl<'a> PlaceExpression<'a> {
    pub(super) fn analyze(expression: &'a Expr) -> Self {
        let mut place = Self {
            authority_fields: Vec::new(),
            direct_field: direct_field(expression),
            excluded_reads: Vec::new(),
        };
        place.collect(expression, true, true);
        place
    }

    pub(super) fn authority_fields(&self) -> impl Iterator<Item = &'a ExprField> + '_ {
        self.authority_fields.iter().copied()
    }

    pub(super) fn is_direct(&self, field: &ExprField) -> bool {
        self.direct_field
            .is_some_and(|direct| std::ptr::eq(direct, field))
    }

    pub(super) fn excluded_reads(&self) -> impl Iterator<Item = SourceSpan> + '_ {
        self.excluded_reads.iter().copied()
    }

    fn collect(&mut self, expression: &'a Expr, authority: bool, exclude_reads: bool) {
        match expression {
            Expr::Field(field) => {
                self.collect_field(field, authority, exclude_reads);
                self.collect(&field.base, authority, exclude_reads);
            }
            Expr::Index(index) => self.collect(&index.expr, authority, exclude_reads),
            Expr::Group(group) => self.collect(&group.expr, authority, exclude_reads),
            Expr::Paren(paren) => self.collect(&paren.expr, authority, exclude_reads),
            Expr::Unary(unary) if matches!(unary.op, UnOp::Deref(_)) => {
                // A pointee write reads the pointer but does not mutate it.
                self.collect(&unary.expr, false, false);
            }
            Expr::Tuple(tuple) => {
                for element in &tuple.elems {
                    self.collect(element, authority, exclude_reads);
                }
            }
            Expr::Array(array) => {
                for element in &array.elems {
                    self.collect(element, authority, exclude_reads);
                }
            }
            _ => {}
        }
    }

    fn collect_field(&mut self, field: &'a ExprField, authority: bool, exclude_reads: bool) {
        let Member::Named(member) = &field.member else {
            return;
        };
        if authority {
            self.authority_fields.push(field);
        }
        if exclude_reads {
            self.excluded_reads.push(source_span(member.span()));
        }
    }
}

fn direct_field(expression: &Expr) -> Option<&ExprField> {
    match expression {
        Expr::Field(field) if matches!(field.member, Member::Named(_)) => Some(field),
        Expr::Group(group) => direct_field(&group.expr),
        Expr::Paren(paren) => direct_field(&paren.expr),
        _ => None,
    }
}
