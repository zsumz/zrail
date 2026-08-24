//! Complex visitor walks preserve mutation, macro, module, and block semantics.

use syn::{
    BinOp, Block, ExprBinary, ItemMod, Macro, Stmt,
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
        visitor.record_field_operation(
            super::super::SourceOperationKind::FieldWrite,
            &expression.left,
        );
        visitor.without_place_field_reads(&expression.left, |visitor| {
            visit::visit_expr_binary(visitor, expression);
        });
    } else {
        visit::visit_expr_binary(visitor, expression);
    }
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
        let items = block.stmts.iter().filter_map(|statement| match statement {
            Stmt::Item(item) => Some(item),
            _ => None,
        });
        visitor.with_import_scope(items, |visitor| visit::visit_block(visitor, block));
    });
}
