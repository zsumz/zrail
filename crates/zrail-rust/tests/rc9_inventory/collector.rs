//! Syntax census candidates require human assertion review; they are never dispositions.

use proc_macro2::Span;
use quote::ToTokens;
use serde::Serialize;
use syn::{spanned::Spanned, visit::Visit};

#[derive(Debug, Serialize)]
pub(super) struct Candidate {
    pub(super) id: String,
    pub(super) kind: &'static str,
    pub(super) function: Vec<String>,
    pub(super) line: usize,
    pub(super) column: usize,
    pub(super) end_line: usize,
    pub(super) end_column: usize,
    pub(super) syntax_sha256: String,
    pub(super) review: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct Function {
    pub(super) name: String,
    pub(super) line: usize,
    pub(super) attributes: Vec<String>,
}

#[derive(Default)]
pub(super) struct Collector {
    pub(super) prefix: String,
    pub(super) candidates: Vec<Candidate>,
    pub(super) functions: Vec<Function>,
    stack: Vec<String>,
}

impl Collector {
    fn record(&mut self, kind: &'static str, span: Span, syntax: impl ToTokens) {
        let start = span.start();
        let end = span.end();
        self.candidates.push(Candidate {
            id: format!("{}:{}:{}:{kind}", self.prefix, start.line, start.column + 1),
            kind,
            function: self.stack.clone(),
            line: start.line,
            column: start.column + 1,
            end_line: end.line,
            end_column: end.column + 1,
            syntax_sha256: zrail_core::sha256_hex(syntax.to_token_stream().to_string().as_bytes()),
            review: "pending",
        });
    }

    fn enter(&mut self, name: &syn::Ident, attributes: &[syn::Attribute]) {
        self.stack.push(name.to_string());
        self.functions.push(Function {
            name: self.stack.join("::"),
            line: name.span().start().line,
            attributes: attributes
                .iter()
                .map(|value| value.to_token_stream().to_string())
                .collect(),
        });
    }
}

impl<'ast> Visit<'ast> for Collector {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.enter(&node.sig.ident, &node.attrs);
        syn::visit::visit_item_fn(self, node);
        self.stack.pop();
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.enter(&node.sig.ident, &node.attrs);
        syn::visit::visit_impl_item_fn(self, node);
        self.stack.pop();
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        self.enter(&node.sig.ident, &node.attrs);
        syn::visit::visit_trait_item_fn(self, node);
        self.stack.pop();
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        let name = node
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_default();
        let kind = match name.as_str() {
            "assert" | "assert_eq" | "assert_ne" => "assertion-macro",
            "panic" | "unreachable" => "failure-macro",
            _ => "macro-review-boundary",
        };
        self.record(kind, node.span(), node);
        // Opaque token input is explicitly inventoried, never parsed as arbitrary Rust.
        syn::visit::visit_macro(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = node.func.as_ref() {
            let name = path
                .path
                .segments
                .last()
                .map(|segment| segment.ident.to_string())
                .unwrap_or_default();
            if ["assert", "check", "reject", "validate", "require"]
                .iter()
                .any(|prefix| name.starts_with(prefix))
            {
                self.record("helper-call-candidate", node.span(), node);
            }
            if fallible_call(&name) {
                let identifier = &path.path.segments.last().expect("named call path").ident;
                self.record("fallible-call-candidate", identifier.span(), node);
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "push" || node.method == "extend" {
            self.record("accumulator-candidate", node.span(), node);
        }
        if fallible_call(&node.method.to_string()) {
            // A chained receiver has the same start span at each call. The method
            // identifier gives every independent precondition a stable location.
            self.record("fallible-call-candidate", node.method.span(), node);
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

fn fallible_call(name: &str) -> bool {
    matches!(
        name.strip_prefix("r#").unwrap_or(name),
        "expect" | "expect_err" | "unwrap" | "unwrap_err"
    )
}
