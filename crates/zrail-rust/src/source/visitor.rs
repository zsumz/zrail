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
    imports::ImportMap,
    model::{IncludeBoundary, ObservedFact},
};

#[derive(Debug)]
pub(super) struct FactVisitor<'a> {
    imports: &'a ImportMap,
    pub(super) paths: Vec<ObservedFact>,
    pub(super) calls: Vec<ObservedFact>,
    pub(super) methods: Vec<ObservedFact>,
    pub(super) macros: Vec<ObservedFact>,
    pub(super) lint_suppressions: Vec<ObservedFact>,
    pub(super) unsafe_constructs: Vec<ObservedFact>,
    pub(super) tests: Vec<ObservedFact>,
    pub(super) includes: Vec<IncludeBoundary>,
    pub(super) item_macros: Vec<ObservedFact>,
}

impl<'a> FactVisitor<'a> {
    pub(super) fn new(imports: &'a ImportMap) -> Self {
        let mut paths = imports
            .declared_paths()
            .into_iter()
            .map(|(path, quality)| ObservedFact {
                name: path.to_owned(),
                span: None,
                quality,
            })
            .collect::<Vec<_>>();
        paths.extend(imports.globs().iter().map(|path| ObservedFact {
            name: path.clone(),
            span: None,
            quality: AnalysisQuality::Conservative,
        }));
        Self {
            imports,
            paths,
            calls: Vec::new(),
            methods: Vec::new(),
            macros: Vec::new(),
            lint_suppressions: Vec::new(),
            unsafe_constructs: Vec::new(),
            tests: Vec::new(),
            includes: Vec::new(),
            item_macros: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for FactVisitor<'_> {
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
        let name = invocation
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_default();
        self.macros
            .push(fact(name, invocation.path.span(), AnalysisQuality::Exact));
        visit::visit_macro(self, invocation);
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
        self.record_expression_macro(&statement.mac);
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
        if let Some(unsafe_token) = &module.unsafety {
            self.unsafe_constructs.push(fact(
                "unsafe module",
                unsafe_token.span,
                AnalysisQuality::Exact,
            ));
        }
        if module.content.is_some() && module.attrs.iter().any(is_cfg_test) {
            self.tests.push(fact(
                format!("inline module {}", module.ident),
                module.ident.span(),
                AnalysisQuality::Exact,
            ));
        }
        visit::visit_item_mod(self, module);
    }
}
