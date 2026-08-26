//! Syntax visitor collecting source facts after import resolution.

mod walk;

use syn::{
    Attribute, Block, ExprBinary, ExprCall, ExprField, ExprForLoop, ExprIf, ExprMacro,
    ExprMethodCall, ExprPath, ExprStruct, ExprWhile, ItemForeignMod, ItemMacro, ItemMod,
    ItemStatic, Macro, PatStruct, StmtMacro, TypePath,
    visit::{self, Visit},
};

use super::{SyntaxGuard, attributes::cfg_guard, visitor_parts::visitor_context};

pub(super) use super::visitor_parts::FactVisitor;

impl<'ast> Visit<'ast> for FactVisitor<'_> {
    fn visit_file(&mut self, file: &'ast syn::File) {
        let guard = cfg_guard(&file.attrs);
        if guard != SyntaxGuard::Ordinary {
            self.guard_initial_paths(&guard);
        }
        self.with_cfg(&file.attrs, |visitor| {
            visitor.with_local_type_scope(file.items.iter(), |visitor| {
                visitor.with_import_scope(file.items.iter(), |visitor| {
                    visit::visit_file(visitor, file);
                });
            });
        });
    }

    fn visit_item(&mut self, item: &'ast syn::Item) {
        self.with_cfg(visitor_context::item_attrs(item), |visitor| {
            visitor.with_fresh_generics(|visitor| visit::visit_item(visitor, item));
        });
    }

    fn visit_impl_item(&mut self, item: &'ast syn::ImplItem) {
        self.with_cfg(visitor_context::impl_attrs(item), |visitor| {
            visit::visit_impl_item(visitor, item);
        });
    }

    fn visit_trait_item(&mut self, item: &'ast syn::TraitItem) {
        self.with_cfg(visitor_context::trait_attrs(item), |visitor| {
            visit::visit_trait_item(visitor, item);
        });
    }

    fn visit_foreign_item(&mut self, item: &'ast syn::ForeignItem) {
        self.with_cfg(visitor_context::foreign_attrs(item), |visitor| {
            visit::visit_foreign_item(visitor, item);
        });
    }

    fn visit_expr(&mut self, expression: &'ast syn::Expr) {
        self.with_cfg(visitor_context::expr_attrs(expression), |visitor| {
            visit::visit_expr(visitor, expression);
        });
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        self.with_cfg(&local.attrs, |visitor| {
            visit::visit_local(visitor, local);
            visitor.record_local_bindings(local);
        });
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        self.with_cfg(&arm.attrs, |visitor| walk::visit_arm(visitor, arm));
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
        self.record_path(path);
        visit::visit_path(self, path);
    }

    fn visit_visibility(&mut self, _: &'ast syn::Visibility) {}

    fn visit_expr_path(&mut self, expression: &'ast ExprPath) {
        self.record_path_construction(expression);
        self.record_expression_path(expression);
    }

    fn visit_type_path(&mut self, path: &'ast TypePath) {
        let previous = std::mem::replace(&mut self.next_path_namespace, super::FactNamespace::Type);
        visit::visit_type_path(self, path);
        self.next_path_namespace = previous;
    }

    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        self.record_attribute(attribute);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        self.record_call_construction(call);
        self.record_call(call);
        visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        self.record_field_receiver_call(call);
        self.record_method_operation(call);
        self.record_method_call(call);
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_struct(&mut self, expression: &'ast ExprStruct) {
        self.record_struct_construction(expression);
        visit::visit_expr_struct(self, expression);
    }

    fn visit_expr_field(&mut self, expression: &'ast ExprField) {
        self.record_field_read(expression);
        visit::visit_expr_field(self, expression);
    }

    fn visit_expr_assign(&mut self, expression: &'ast syn::ExprAssign) {
        walk::visit_assign(self, expression);
    }

    fn visit_expr_binary(&mut self, expression: &'ast ExprBinary) {
        walk::visit_binary(self, expression);
    }

    fn visit_expr_reference(&mut self, expression: &'ast syn::ExprReference) {
        walk::visit_reference(self, expression);
    }

    fn visit_expr_async(&mut self, expression: &'ast syn::ExprAsync) {
        walk::visit_async(self, expression);
    }

    fn visit_expr_closure(&mut self, expression: &'ast syn::ExprClosure) {
        walk::visit_closure(self, expression);
    }

    fn visit_expr_if(&mut self, expression: &'ast ExprIf) {
        walk::visit_if(self, expression);
    }

    fn visit_expr_while(&mut self, expression: &'ast ExprWhile) {
        walk::visit_while(self, expression);
    }

    fn visit_expr_for_loop(&mut self, expression: &'ast ExprForLoop) {
        walk::visit_for(self, expression);
    }

    fn visit_expr_await(&mut self, expression: &'ast syn::ExprAwait) {
        walk::visit_await(self, expression);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        walk::visit_macro(self, invocation);
    }

    fn visit_item_macro(&mut self, item: &'ast ItemMacro) {
        self.record_item_macro(item);
        visit::visit_item_macro(self, item);
    }

    fn visit_expr_macro(&mut self, expression: &'ast ExprMacro) {
        self.record_expression_macro(&expression.mac);
        visit::visit_expr_macro(self, expression);
    }

    fn visit_stmt_macro(&mut self, statement: &'ast StmtMacro) {
        self.with_cfg(&statement.attrs, |visitor| {
            visitor.record_statement_macro(statement);
            visit::visit_stmt_macro(visitor, statement);
        });
    }

    fn visit_expr_unsafe(&mut self, expression: &'ast syn::ExprUnsafe) {
        self.record_unsafe_expression(expression);
        visit::visit_expr_unsafe(self, expression);
    }

    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        walk::visit_item_fn(self, function);
    }

    fn visit_signature(&mut self, signature: &'ast syn::Signature) {
        walk::visit_signature(self, signature);
    }

    fn visit_item_impl(&mut self, implementation: &'ast syn::ItemImpl) {
        walk::visit_item_impl(self, implementation);
    }

    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        walk::visit_item_trait(self, item);
    }

    fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
        walk::visit_impl_item_fn(self, function);
    }

    fn visit_trait_item_fn(&mut self, function: &'ast syn::TraitItemFn) {
        walk::visit_trait_item_fn(self, function);
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
        walk::visit_module(self, module);
    }

    fn visit_pat_struct(&mut self, pattern: &'ast PatStruct) {
        self.record_struct_pattern(pattern);
        self.with_pattern_type_paths(|visitor| visit::visit_pat_struct(visitor, pattern));
    }

    fn visit_block(&mut self, block: &'ast Block) {
        walk::visit_block(self, block);
    }
}
