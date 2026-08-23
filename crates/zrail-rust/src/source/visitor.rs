//! Syntax visitor collecting source facts after import resolution.

use syn::{
    Attribute, Block, ExprCall, ExprMacro, ExprMethodCall, ItemFn, ItemForeignMod, ItemImpl,
    ItemMacro, ItemMod, ItemStatic, ItemTrait, Macro, Signature, Stmt, StmtMacro,
    spanned::Spanned,
    visit::{self, Visit},
};
use zrail_core::AnalysisQuality;

use super::{
    attributes::{is_cfg_test, is_test_attribute},
    fact::fact,
    visitor_context::{expr_attrs, foreign_attrs, impl_attrs, item_attrs, trait_attrs},
};

pub(super) use super::visitor_model::FactVisitor;

impl<'ast> Visit<'ast> for FactVisitor<'_> {
    fn visit_file(&mut self, file: &'ast syn::File) {
        if file.attrs.iter().any(is_cfg_test) {
            self.guard_initial_paths();
        }
        self.with_cfg(&file.attrs, |visitor| {
            visitor.with_import_scope(file.items.iter(), |visitor| {
                visit::visit_file(visitor, file);
            });
        });
    }

    fn visit_item(&mut self, item: &'ast syn::Item) {
        self.with_cfg(item_attrs(item), |visitor| visit::visit_item(visitor, item));
    }

    fn visit_impl_item(&mut self, item: &'ast syn::ImplItem) {
        self.with_cfg(impl_attrs(item), |visitor| {
            visit::visit_impl_item(visitor, item);
        });
    }

    fn visit_trait_item(&mut self, item: &'ast syn::TraitItem) {
        self.with_cfg(trait_attrs(item), |visitor| {
            visit::visit_trait_item(visitor, item);
        });
    }

    fn visit_foreign_item(&mut self, item: &'ast syn::ForeignItem) {
        self.with_cfg(foreign_attrs(item), |visitor| {
            visit::visit_foreign_item(visitor, item);
        });
    }

    fn visit_expr(&mut self, expression: &'ast syn::Expr) {
        self.with_cfg(expr_attrs(expression), |visitor| {
            visit::visit_expr(visitor, expression);
        });
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        self.with_cfg(&local.attrs, |visitor| visit::visit_local(visitor, local));
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        self.with_cfg(&arm.attrs, |visitor| visit::visit_arm(visitor, arm));
    }

    fn visit_field(&mut self, field: &'ast syn::Field) {
        self.with_cfg(&field.attrs, |visitor| visit::visit_field(visitor, field));
    }

    fn visit_variant(&mut self, variant: &'ast syn::Variant) {
        self.with_cfg(&variant.attrs, |visitor| {
            visit::visit_variant(visitor, variant);
        });
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        let guard = self.syntax_guard();
        let (name, quality) = self.imports.resolve(path, guard);
        if !name.is_empty() {
            self.paths.push(fact(name.as_str(), path.span(), quality));
            self.paths
                .extend(super::calls::candidates(path, self.imports, &name, guard));
        }
        visit::visit_path(self, path);
    }

    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        self.record_attribute(attribute);
        visit::visit_attribute(self, attribute);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        self.calls
            .extend(super::calls::facts(call, self.imports, self.syntax_guard()));
        visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        self.methods.push(fact(
            call.method.to_string(),
            call.method.span(),
            AnalysisQuality::Conservative,
        ));
        visit::visit_expr_method_call(self, call);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        if invocation.path.is_ident("macro_rules") {
            visit::visit_macro(self, invocation);
            return;
        }
        let expansion = self.macro_invocation(&invocation.path);
        super::compile_effects::record(self, invocation, &expansion);
        let opaque_input = super::macro_inputs::inspect(self, invocation, &expansion.name);
        self.macros.extend(
            expansion
                .candidates
                .iter()
                .map(|fact| fact.observation.clone()),
        );
        self.macro_expansions.push(expansion.clone());
        if opaque_input {
            self.opaque_macro_inputs.push(expansion);
        }
        visit::visit_macro(self, invocation);
    }

    fn visit_item_macro(&mut self, item: &'ast ItemMacro) {
        self.record_item_macro(item);
        visit::visit_item_macro(self, item);
    }

    fn visit_expr_macro(&mut self, expression: &'ast ExprMacro) {
        self.record_expression_macro(&expression.mac, expression.attrs.iter().any(is_cfg_test));
        visit::visit_expr_macro(self, expression);
    }

    fn visit_stmt_macro(&mut self, statement: &'ast StmtMacro) {
        self.with_cfg(&statement.attrs, |visitor| {
            visitor.record_statement_macro(statement);
            visit::visit_stmt_macro(visitor, statement);
        });
    }

    fn visit_expr_unsafe(&mut self, expression: &'ast syn::ExprUnsafe) {
        self.unsafe_constructs.push(fact(
            "unsafe block",
            expression.unsafe_token.span,
            AnalysisQuality::Exact,
        ));
        visit::visit_expr_unsafe(self, expression);
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        if function.attrs.iter().any(is_test_attribute) {
            self.tests.push(fact(
                function.sig.ident.to_string(),
                function.sig.ident.span(),
                AnalysisQuality::Exact,
            ));
        }
        visit::visit_item_fn(self, function);
    }

    fn visit_signature(&mut self, signature: &'ast Signature) {
        if signature.unsafety.is_some() {
            self.unsafe_constructs.push(fact(
                "unsafe function",
                signature.span(),
                AnalysisQuality::Exact,
            ));
        }
        visit::visit_signature(self, signature);
    }

    fn visit_item_impl(&mut self, implementation: &'ast ItemImpl) {
        if implementation.unsafety.is_some() {
            self.unsafe_constructs.push(fact(
                "unsafe impl",
                implementation.impl_token.span,
                AnalysisQuality::Exact,
            ));
        }
        visit::visit_item_impl(self, implementation);
    }

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        if item.unsafety.is_some() {
            self.unsafe_constructs.push(fact(
                "unsafe trait",
                item.trait_token.span,
                AnalysisQuality::Exact,
            ));
        }
        visit::visit_item_trait(self, item);
    }

    fn visit_item_foreign_mod(&mut self, item: &'ast ItemForeignMod) {
        self.record_foreign_mod(item);
        visit::visit_item_foreign_mod(self, item);
    }

    fn visit_item_static(&mut self, item: &'ast ItemStatic) {
        self.record_static(item);
        visit::visit_item_static(self, item);
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.record_module(module);
        for attribute in &module.attrs {
            self.visit_attribute(attribute);
        }
        self.visit_visibility(&module.vis);
        if let Some((_, items)) = &module.content {
            self.with_lexical_scope(module.ident.span(), |visitor| {
                visitor.with_import_scope(items.iter(), |visitor| {
                    for item in items {
                        visitor.visit_item(item);
                    }
                });
            });
        }
    }

    fn visit_block(&mut self, block: &'ast Block) {
        self.with_lexical_scope(block.span(), |visitor| {
            visitor.with_import_scope(
                block.stmts.iter().filter_map(|statement| match statement {
                    Stmt::Item(item) => Some(item),
                    _ => None,
                }),
                |visitor| visit::visit_block(visitor, block),
            );
        });
    }
}
