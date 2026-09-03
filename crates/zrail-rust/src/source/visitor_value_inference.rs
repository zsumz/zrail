//! Untyped locals retain bounded associated-return receipts for later proof.

use super::{FactVisitor, ValueBinding, candidates::binding_from_identity};
use crate::source::{
    FactNamespace, RootLookupNamespace,
    fact::written_fact,
    generic_root_shadow,
    operation_model::{AssociatedReturnInference, subject::WrittenOperationSubject, unresolved},
};
use syn::{Expr, spanned::Spanned};

const MAX_TRY_DEPTH: usize = 8;

pub(super) fn binding(visitor: &FactVisitor<'_>, expression: &Expr) -> Option<ValueBinding> {
    let (expression, try_depth) = peel_try(expression)?;
    let Expr::Call(call) = expression else {
        return None;
    };
    let Expr::Path(callee) = transparent(&call.func) else {
        return None;
    };
    if callee.qself.is_some() || callee.path.segments.len() < 2 {
        return None;
    }
    let subject = WrittenOperationSubject::from_expression(callee);
    let path = subject.construction_path()?;
    let written = subject.written();
    let root_lookup = subject.root_lookup();
    if root_lookup != RootLookupNamespace::Type {
        return None;
    }
    let identity = visitor.resolve_construction_identity(&path);
    let mut fact = written_fact(
        identity.name.clone(),
        written.clone(),
        callee.path.span(),
        identity.quality,
        &visitor.lexical_scope,
    );
    fact.apply_guard(&visitor.syntax_guard());
    fact.inherits_parent_context = visitor.inherits_parent_context;
    fact.namespace = FactNamespace::Type;
    let generic_shadow = generic_root_shadow(
        &written,
        root_lookup,
        &visitor.generic_types,
        &visitor.generic_values,
    );
    let mut inferred = unresolved("<unresolved>");
    inferred.inference = Some(AssociatedReturnInference {
        fact,
        subject_origin: identity.origin,
        root_lookup,
        generic_shadow,
        try_depth,
    });
    Some(binding_from_identity(inferred))
}

fn peel_try(mut expression: &Expr) -> Option<(&Expr, usize)> {
    let mut depth = 0;
    loop {
        expression = transparent(expression);
        let Expr::Try(value) = expression else {
            return (depth <= MAX_TRY_DEPTH).then_some((expression, depth));
        };
        depth += 1;
        expression = &value.expr;
    }
}

fn transparent(mut expression: &Expr) -> &Expr {
    loop {
        expression = match expression {
            Expr::Group(group) => &group.expr,
            Expr::Paren(paren) => &paren.expr,
            _ => return expression,
        };
    }
}
