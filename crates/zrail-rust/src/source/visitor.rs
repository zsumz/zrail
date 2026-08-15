//! Syntax visitor collecting source facts after import resolution.

use syn::{
    Attribute, ExprCall, ExprMacro, ExprMethodCall, ItemFn, ItemForeignMod, ItemImpl, ItemMacro,
    ItemMod, ItemStatic, ItemTrait, Macro, Signature, StmtMacro,
    spanned::Spanned,
    visit::{self, Visit},
};
use zrail_core::AnalysisQuality;

use super::{
    attributes::{
        is_cfg_test, is_lint_suppression, is_test_attribute, lint_suppression_is_reasoned,
        unsafe_attribute_names,
    },
    fact::fact,
    visitor_context::{expr_attrs, foreign_attrs, impl_attrs, item_attrs, trait_attrs},
};

pub(super) use super::visitor_model::FactVisitor;

impl<'ast> Visit<'ast> for FactVisitor<'_> {
    fn visit_file(&mut self, file: &'ast syn::File) {
        self.with_cfg(&file.attrs, |visitor| visit::visit_file(visitor, file));
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
        let (name, quality) = self.imports.resolve(path);
        if !name.is_empty() {
            self.paths.push(fact(name.as_str(), path.span(), quality));
            self.paths
                .extend(super::calls::candidates(path, self.imports, &name));
        }
        visit::visit_path(self, path);
    }

    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        if is_lint_suppression(attribute) {
            self.lint_suppressions.push(fact(
                if lint_suppression_is_reasoned(attribute) {
                    "reasoned lint suppression"
                } else {
                    "unreasoned lint suppression"
                },
                attribute.span(),
                AnalysisQuality::Exact,
            ));
        }
        let attribute_quality = if attribute.path().is_ident("cfg_attr") {
            AnalysisQuality::Conservative
        } else {
            AnalysisQuality::Exact
        };
        self.unsafe_constructs
            .extend(unsafe_attribute_names(attribute).into_iter().map(|name| {
                fact(
                    format!("unsafe attribute {name}"),
                    attribute.span(),
                    attribute_quality,
                )
            }));
        visit::visit_attribute(self, attribute);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        self.calls.extend(super::calls::facts(call, self.imports));
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
        let (name, quality) = self.imports.resolve(&invocation.path);
        self.macros
            .push(fact(name.clone(), invocation.path.span(), quality));
        self.macros.extend(super::calls::candidates(
            &invocation.path,
            self.imports,
            &name,
        ));
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
        self.record_expression_macro(&statement.mac, statement.attrs.iter().any(is_cfg_test));
        visit::visit_stmt_macro(self, statement);
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
        self.unsafe_constructs.push(fact(
            if item.unsafety.is_some() {
                "unsafe extern block"
            } else {
                "extern block"
            },
            item.abi.extern_token.span,
            AnalysisQuality::Exact,
        ));
        visit::visit_item_foreign_mod(self, item);
    }

    fn visit_item_static(&mut self, item: &'ast ItemStatic) {
        if let syn::StaticMutability::Mut(mut_token) = &item.mutability {
            self.unsafe_constructs.push(fact(
                "mutable static",
                mut_token.span,
                AnalysisQuality::Exact,
            ));
        }
        visit::visit_item_static(self, item);
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.record_module(module);
        visit::visit_item_mod(self, module);
    }
}
