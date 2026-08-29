//! Complex visitor walks preserve mutation, macro, module, and block semantics.

mod async_syntax;
pub(super) use async_syntax::{visit_async, visit_await};
mod mutation;
pub(super) use mutation::{visit_assign, visit_binary, visit_raw_address, visit_reference};

use syn::{
    Block, ExprClosure, ExprForLoop, ImplItemFn, Item, ItemFn, ItemImpl, ItemMod, ItemTrait, Macro,
    Signature, Stmt, TraitItemFn,
    spanned::Spanned,
    visit::{self, Visit},
};

use super::super::visitor_parts::visitor_patterns::PatternInputMode;
use super::FactVisitor;

pub(super) fn visit_item(visitor: &mut FactVisitor<'_>, item: &Item) {
    let local_values = std::mem::take(&mut visitor.local_values);
    let pattern_inputs = std::mem::take(&mut visitor.pattern_inputs);
    let self_types = std::mem::take(&mut visitor.self_types);
    let inherits_parent_context = visitor.inherits_parent_context;
    if visitor.block_depth > 0 {
        visitor.inherits_parent_context = false;
    }
    visitor.with_fresh_generics(|visitor| visit::visit_item(visitor, item));
    visitor.local_values = local_values;
    visitor.pattern_inputs = pattern_inputs;
    visitor.self_types = self_types;
    visitor.inherits_parent_context = inherits_parent_context;
}

pub(super) fn visit_closure(visitor: &mut FactVisitor<'_>, expression: &ExprClosure) {
    if let Some(token) = expression.asyncness {
        visitor.record_async_syntax(zrail_core::AsyncSyntax::AsyncClosure, token.span);
    }
    visitor.with_closure_values(&expression.inputs, |visitor| {
        visit::visit_expr_closure(visitor, expression);
    });
}

pub(super) fn visit_for(visitor: &mut FactVisitor<'_>, expression: &ExprForLoop) {
    visitor.visit_expr(&expression.expr);
    let input = PatternInputMode::Unresolved;
    visitor.with_pattern_input(input, |visitor| visitor.visit_pat(&expression.pat));
    visitor.with_pattern_values(&expression.pat, input, |visitor| {
        visitor.visit_block(&expression.body);
    });
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
    let mut self_bounds = implementation
        .trait_
        .as_ref()
        .filter(|(negative, _, _)| negative.is_none())
        .map(|(_, path, _)| {
            super::super::trait_bounds::current_trait_bounds(
                super::super::GenericPathIdentity::trait_path(path),
                &visitor.syntax_guard(),
                &visitor.lexical_scope,
                super::super::fact::source_span(path.span()),
            )
        })
        .unwrap_or_default();
    self_bounds.extend(super::super::trait_bounds::impl_associated_types(
        implementation,
        &visitor.syntax_guard(),
        &visitor.lexical_scope,
    ));
    visitor.with_self_type(&implementation.self_ty, |visitor| {
        visitor.with_generics_and_bounds(&implementation.generics, true, self_bounds, |visitor| {
            visit::visit_item_impl(visitor, implementation);
        });
    });
}

pub(super) fn visit_item_trait(visitor: &mut FactVisitor<'_>, item: &ItemTrait) {
    visitor.record_unsafe_trait(item);
    let mut bounds = super::super::trait_bounds::current_trait_bounds(
        super::super::GenericPathIdentity::wildcard(item.ident.to_string()),
        &visitor.syntax_guard(),
        &visitor.lexical_scope,
        super::super::fact::source_span(item.ident.span()),
    );
    bounds.extend(super::super::trait_bounds::associated_types(
        item,
        &visitor.syntax_guard(),
        &visitor.lexical_scope,
    ));
    visitor.with_generics_and_bounds(&item.generics, true, bounds, |visitor| {
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
    visitor.block_depth = visitor.block_depth.saturating_add(1);
    visitor.with_lexical_scope(block.span(), |visitor| {
        visitor.with_value_scope(|visitor| {
            let items = block.stmts.iter().filter_map(|statement| match statement {
                Stmt::Item(item) => Some(item),
                _ => None,
            });
            visitor.with_import_scope(items, |visitor| visit::visit_block(visitor, block));
        });
    });
    visitor.block_depth = visitor.block_depth.saturating_sub(1);
}
