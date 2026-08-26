//! Complex visitor walks preserve mutation, macro, module, and block semantics.

use syn::{
    BinOp, Block, ExprAssign, ExprAsync, ExprAwait, ExprBinary, ExprClosure, ExprReference,
    ImplItemFn, ItemFn, ItemImpl, ItemMod, ItemTrait, Macro, Signature, Stmt, TraitItemFn,
    spanned::Spanned,
    visit::{self, Visit},
};

use super::FactVisitor;

pub(super) fn visit_binary(visitor: &mut FactVisitor<'_>, expression: &ExprBinary) {
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
            super::super::SourceOperationKind::FieldWrite,
            &expression.left,
            |visitor| {
                visit::visit_expr_binary(visitor, expression);
            },
        );
    } else {
        visit::visit_expr_binary(visitor, expression);
    }
}

pub(super) fn visit_assign(visitor: &mut FactVisitor<'_>, expression: &ExprAssign) {
    visitor.with_place_operation(
        super::super::SourceOperationKind::FieldWrite,
        &expression.left,
        |visitor| visit::visit_expr_assign(visitor, expression),
    );
}

pub(super) fn visit_reference(visitor: &mut FactVisitor<'_>, expression: &ExprReference) {
    if expression.mutability.is_some() {
        visitor.with_place_operation(
            super::super::SourceOperationKind::FieldMutableBorrow,
            &expression.expr,
            |visitor| visit::visit_expr_reference(visitor, expression),
        );
    } else {
        visit::visit_expr_reference(visitor, expression);
    }
}

pub(super) fn visit_async(visitor: &mut FactVisitor<'_>, expression: &ExprAsync) {
    visitor.record_async_syntax(
        zrail_core::AsyncSyntax::AsyncBlock,
        expression.async_token.span,
    );
    visit::visit_expr_async(visitor, expression);
}

pub(super) fn visit_closure(visitor: &mut FactVisitor<'_>, expression: &ExprClosure) {
    if let Some(token) = expression.asyncness {
        visitor.record_async_syntax(zrail_core::AsyncSyntax::AsyncClosure, token.span);
    }
    visitor.with_closure_values(&expression.inputs, |visitor| {
        visit::visit_expr_closure(visitor, expression);
    });
}

pub(super) fn visit_await(visitor: &mut FactVisitor<'_>, expression: &ExprAwait) {
    visitor.record_async_syntax(zrail_core::AsyncSyntax::Await, expression.await_token.span);
    visit::visit_expr_await(visitor, expression);
}

pub(super) fn visit_item_fn(visitor: &mut FactVisitor<'_>, function: &ItemFn) {
    visitor.record_test_function(function);
    visitor.with_generics(&function.sig.generics, false, |visitor| {
        visitor.with_signature_values(&function.sig, |visitor| {
            visit::visit_item_fn(visitor, function);
        });
    });
}

pub(super) fn visit_signature(visitor: &mut FactVisitor<'_>, signature: &Signature) {
    if let Some(token) = signature.asyncness {
        visitor.record_async_syntax(zrail_core::AsyncSyntax::AsyncFn, token.span);
    }
    visitor.record_unsafe_signature(signature);
    visit::visit_signature(visitor, signature);
}

pub(super) fn visit_item_impl(visitor: &mut FactVisitor<'_>, implementation: &ItemImpl) {
    visitor.record_unsafe_impl(implementation);
    visitor.with_self_type(&implementation.self_ty, |visitor| {
        visitor.with_generics(&implementation.generics, true, |visitor| {
            visit::visit_item_impl(visitor, implementation);
        });
    });
}

pub(super) fn visit_item_trait(visitor: &mut FactVisitor<'_>, item: &ItemTrait) {
    visitor.record_unsafe_trait(item);
    visitor.with_generics(&item.generics, true, |visitor| {
        visit::visit_item_trait(visitor, item);
    });
}

pub(super) fn visit_impl_item_fn(visitor: &mut FactVisitor<'_>, function: &ImplItemFn) {
    visitor.with_generics(&function.sig.generics, false, |visitor| {
        visitor.with_signature_values(&function.sig, |visitor| {
            visit::visit_impl_item_fn(visitor, function);
        });
    });
}

pub(super) fn visit_trait_item_fn(visitor: &mut FactVisitor<'_>, function: &TraitItemFn) {
    visitor.with_generics(&function.sig.generics, false, |visitor| {
        visitor.with_signature_values(&function.sig, |visitor| {
            visit::visit_trait_item_fn(visitor, function);
        });
    });
}

pub(super) fn visit_macro(visitor: &mut FactVisitor<'_>, invocation: &Macro) {
    if invocation.path.is_ident("macro_rules") {
        return;
    }
    let expansion = visitor
        .macro_invocation(&invocation.path)
        .with_input_tokens(&invocation.tokens);
    super::super::compile_effects::record(visitor, invocation, &expansion);
    let opaque_input = super::super::macro_inputs::inspect(visitor, invocation, &expansion.name);
    visitor.macros.extend(
        expansion
            .candidates
            .iter()
            .map(|fact| fact.observation.clone()),
    );
    visitor.macro_expansions.push(expansion.clone());
    if opaque_input {
        visitor.opaque_macro_inputs.push(expansion);
    }
}

pub(super) fn visit_module(visitor: &mut FactVisitor<'_>, module: &ItemMod) {
    visitor.record_module(module);
    for attribute in &module.attrs {
        visitor.visit_attribute(attribute);
    }
    visitor.visit_visibility(&module.vis);
    if let Some((_, items)) = &module.content {
        visitor.with_lexical_scope(module.ident.span(), |visitor| {
            visitor.with_inline_module(module.ident.to_string(), |visitor| {
                visitor.with_local_type_scope(items.iter(), |visitor| {
                    visitor.with_import_scope(items.iter(), |visitor| {
                        for item in items {
                            visitor.visit_item(item);
                        }
                    });
                });
            });
        });
    }
}

pub(super) fn visit_block(visitor: &mut FactVisitor<'_>, block: &Block) {
    visitor.with_lexical_scope(block.span(), |visitor| {
        visitor.with_value_scope(|visitor| {
            let items = block.stmts.iter().filter_map(|statement| match statement {
                Stmt::Item(item) => Some(item),
                _ => None,
            });
            visitor.with_import_scope(items, |visitor| visit::visit_block(visitor, block));
        });
    });
}
