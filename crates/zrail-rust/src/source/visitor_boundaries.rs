//! Macro positions identify includes and unresolved item-producing expansion.

use syn::{ItemForeignMod, ItemMacro, ItemMod, ItemStatic, Macro, StmtMacro, spanned::Spanned};
use zrail_core::{AnalysisQuality, sha256_hex};

use super::{
    attributes::is_cfg_test,
    fact::{fact, source_span},
    includes::include_boundary,
    model::{IncludeContext, MacroDefinitionFact},
    visitor::FactVisitor,
};

impl FactVisitor<'_> {
    pub(super) fn record_foreign_mod(&mut self, item: &ItemForeignMod) {
        self.unsafe_constructs.push(fact(
            if item.unsafety.is_some() {
                "unsafe extern block"
            } else {
                "extern block"
            },
            item.abi.extern_token.span,
            AnalysisQuality::Exact,
        ));
    }

    pub(super) fn record_static(&mut self, item: &ItemStatic) {
        if let syn::StaticMutability::Mut(mut_token) = &item.mutability {
            self.unsafe_constructs.push(fact(
                "mutable static",
                mut_token.span,
                AnalysisQuality::Exact,
            ));
        }
    }

    pub(super) fn record_module(&mut self, module: &ItemMod) {
        if let Some(unsafe_token) = &module.unsafety {
            self.unsafe_constructs.push(fact(
                "unsafe module",
                unsafe_token.span,
                AnalysisQuality::Exact,
            ));
        }
        let cfg_test = module.content.is_some() && module.attrs.iter().any(is_cfg_test);
        if cfg_test {
            self.tests.push(fact(
                format!("inline module {}", module.ident),
                module.ident.span(),
                AnalysisQuality::Exact,
            ));
        }
    }

    pub(super) fn record_item_macro(&mut self, item: &ItemMacro) {
        if let Some(name) = &item.ident {
            self.macro_definitions.push(MacroDefinitionFact {
                name: name.to_string(),
                sha256: sha256_hex(item.mac.tokens.to_string().as_bytes()),
                span: Some(source_span(name.span())),
                guard: self.syntax_guard(),
                lexical_scope: self.lexical_scope.clone(),
            });
        }
        if let Some(mut boundary) = include_boundary(&item.mac, IncludeContext::Items) {
            boundary.cfg_test = self.test_only_context || item.attrs.iter().any(is_cfg_test);
            boundary.lexical_scope.clone_from(&self.lexical_scope);
            self.includes.push(boundary);
        } else if item.ident.is_none() {
            let (name, _) = self.imports.resolve(&item.mac.path, self.syntax_guard());
            self.item_macros.push(fact(
                name,
                item.mac.path.span(),
                AnalysisQuality::Unresolved,
            ));
        }
    }

    pub(super) fn record_expression_macro(&mut self, invocation: &Macro, cfg_test: bool) {
        if let Some(mut boundary) = include_boundary(invocation, IncludeContext::Expression) {
            boundary.cfg_test = self.test_only_context || cfg_test;
            boundary.lexical_scope.clone_from(&self.lexical_scope);
            self.includes.push(boundary);
        }
    }

    pub(super) fn record_statement_macro(&mut self, statement: &StmtMacro) {
        if statement.mac.path.is_ident("macro_rules") {
            if let Some(proc_macro2::TokenTree::Ident(name)) =
                statement.mac.tokens.clone().into_iter().next()
            {
                self.macro_definitions.push(MacroDefinitionFact {
                    name: name.to_string(),
                    sha256: sha256_hex(statement.mac.tokens.to_string().as_bytes()),
                    span: Some(source_span(name.span())),
                    guard: self.syntax_guard(),
                    lexical_scope: self.lexical_scope.clone(),
                });
            }
            return;
        }
        self.record_expression_macro(&statement.mac, statement.attrs.iter().any(is_cfg_test));
    }
}
