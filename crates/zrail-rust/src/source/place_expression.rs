//! Structural Rust place projections keep mutation authority separate from value reads.

use syn::{Expr, ExprField, Member, UnOp};
use zrail_core::SourceSpan;

use super::fact::source_span;

pub(super) struct PlaceExpression<'a> {
    authority_fields: Vec<&'a ExprField>,
    excluded_reads: Vec<SourceSpan>,
}

impl<'a> PlaceExpression<'a> {
    pub(super) fn analyze(expression: &'a Expr) -> Self {
        let mut place = Self {
            authority_fields: Vec::new(),
            excluded_reads: Vec::new(),
        };
        place.collect(expression, true);
        place
    }

    pub(super) fn authority_fields(&self) -> impl Iterator<Item = &'a ExprField> + '_ {
        self.authority_fields.iter().copied()
    }

    pub(super) fn excluded_reads(&self) -> impl Iterator<Item = SourceSpan> + '_ {
        self.excluded_reads.iter().copied()
    }

    fn collect(&mut self, expression: &'a Expr, exclude_reads: bool) {
        match expression {
            Expr::Field(field) => {
                self.collect_field(field, exclude_reads);
                self.collect(&field.base, exclude_reads);
            }
            Expr::Index(index) => self.collect(&index.expr, exclude_reads),
            Expr::Group(group) => self.collect(&group.expr, exclude_reads),
            Expr::Paren(paren) => self.collect(&paren.expr, exclude_reads),
            Expr::Unary(unary) if matches!(unary.op, UnOp::Deref(_)) => {
                self.collect(&unary.expr, false);
            }
            Expr::Tuple(tuple) => {
                for element in &tuple.elems {
                    self.collect(element, exclude_reads);
                }
            }
            Expr::Array(array) => {
                for element in &array.elems {
                    self.collect(element, exclude_reads);
                }
            }
            _ => {}
        }
    }

    fn collect_field(&mut self, field: &'a ExprField, exclude_reads: bool) {
        let Member::Named(member) = &field.member else {
            return;
        };
        self.authority_fields.push(field);
        if exclude_reads {
            self.excluded_reads.push(source_span(member.span()));
        }
    }
}
