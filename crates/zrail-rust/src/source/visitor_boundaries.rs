//! Macro positions identify includes and unresolved item-producing expansion.

use syn::{
    ExprUnsafe, ItemForeignMod, ItemImpl, ItemMacro, ItemMod, ItemStatic, ItemTrait, Macro,
    Signature, StmtMacro, spanned::Spanned,
};
use zrail_core::{AnalysisQuality, sha256_hex};

use super::{
    FactVisitor,
    fact::{fact, source_span},
    includes::include_boundary,
    model::{IncludeContext, MacroDefinitionFact},
};

impl FactVisitor<'_> {
    pub(in crate::source) fn record_unsafe_expression(&mut self, expression: &ExprUnsafe) {
        self.unsafe_constructs.push(fact(
            "unsafe block",
            expression.unsafe_token.span,
            AnalysisQuality::Exact,
        ));
    }

    pub(in crate::source) fn record_unsafe_signature(&mut self, signature: &Signature) {
        if signature.unsafety.is_some() {
            self.unsafe_constructs.push(fact(
                "unsafe function",
                signature.span(),
                AnalysisQuality::Exact,
            ));
        }
    }

    pub(in crate::source) fn record_unsafe_impl(&mut self, implementation: &ItemImpl) {
        if implementation.unsafety.is_some() {
            self.unsafe_constructs.push(fact(
                "unsafe impl",
                implementation.impl_token.span,
                AnalysisQuality::Exact,
            ));
        }
    }

    pub(in crate::source) fn record_unsafe_trait(&mut self, item: &ItemTrait) {
        if item.unsafety.is_some() {
            self.unsafe_constructs.push(fact(
                "unsafe trait",
                item.trait_token.span,
                AnalysisQuality::Exact,
            ));
        }
    }

    pub(in crate::source) fn record_foreign_mod(&mut self, item: &ItemForeignMod) {
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

    pub(in crate::source) fn record_static(&mut self, item: &ItemStatic) {
        if let syn::StaticMutability::Mut(mut_token) = &item.mutability {
            self.unsafe_constructs.push(fact(
                "mutable static",
                mut_token.span,
                AnalysisQuality::Exact,
            ));
        }
    }

    pub(in crate::source) fn record_module(&mut self, module: &ItemMod) {
        if let Some(unsafe_token) = &module.unsafety {
            self.unsafe_constructs.push(fact(
                "unsafe module",
                unsafe_token.span,
                AnalysisQuality::Exact,
            ));
        }
        let cfg_test = module.content.is_some() && self.syntax_guard().is_test_only();
        if cfg_test {
            self.tests.push(fact(
                format!("inline module {}", module.ident),
                module.ident.span(),
                AnalysisQuality::Exact,
            ));
        }
    }

    pub(in crate::source) fn record_item_macro(&mut self, item: &ItemMacro) {
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
            boundary.guard = self.syntax_guard();
            boundary.lexical_scope.clone_from(&self.lexical_scope);
            self.includes.push(boundary);
        } else if item.ident.is_none() {
            let guard = self.syntax_guard();
            let (name, _) = self.imports.resolve(&item.mac.path, &guard);
            let mut boundary = fact(name, item.mac.path.span(), AnalysisQuality::Unresolved);
            boundary.lexical_scope.clone_from(&self.lexical_scope);
            self.item_macros.push(boundary);
        }
    }

    pub(in crate::source) fn record_expression_macro(&mut self, invocation: &Macro) {
        if let Some(mut boundary) = include_boundary(invocation, IncludeContext::Expression) {
            boundary.guard = self.syntax_guard();
            boundary.lexical_scope.clone_from(&self.lexical_scope);
            boundary.generic_types.clone_from(&self.generic_types);
            boundary.prelude_value_shadows = self.implicit_prelude_value_shadows();
            self.includes.push(boundary);
        }
    }

    pub(in crate::source) fn record_statement_macro(&mut self, statement: &StmtMacro) {
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
        let include_count = self.includes.len();
        self.record_expression_macro(&statement.mac);
        if self.includes.len() == include_count {
            let (name, quality, _, local_module) = self.resolve_macro_path(&statement.mac.path);
            if quality == AnalysisQuality::Exact
                && !local_module
                && super::macro_origins::compiler_builtin(&name)
            {
                return;
            }
            let mut opaque = fact(name, statement.mac.path.span(), AnalysisQuality::Unresolved);
            opaque.lexical_scope.clone_from(&self.lexical_scope);
            self.opaque_binding_macros.push(opaque);
        }
    }
}
