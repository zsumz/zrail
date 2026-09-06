//! Authored AST membership excludes macro-token observations without altering rc8 method facts.
//!
//! Reuse existing located method facts where present. The bounded supplement covers
//! expression contexts intentionally omitted by semantic traversal, such as attributes.

use std::collections::BTreeMap;

use syn::visit::{self, Visit};
use zrail_core::{AnalysisQuality, SourceSpan};

use super::{
    ObservedFact,
    fact::{fact, source_span},
};

pub(super) fn collect(syntax: &syn::File, methods: &[ObservedFact]) -> Vec<ObservedFact> {
    let existing = methods
        .iter()
        .filter_map(|fact| fact.span.map(|span| ((span, fact.name.as_str()), fact)))
        .collect();
    let mut visitor = AuthoredMethods {
        existing,
        methods: Vec::new(),
    };
    visitor.visit_file(syntax);
    visitor.methods
}

struct AuthoredMethods<'a> {
    existing: BTreeMap<(SourceSpan, &'a str), &'a ObservedFact>,
    methods: Vec<ObservedFact>,
}

impl<'ast> Visit<'ast> for AuthoredMethods<'_> {
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        // Preserve one excess observation so the existing parser fact limit rejects it.
        if self.methods.len() > super::parse::MAX_FACTS_PER_FILE {
            return;
        }
        let name = call.method.to_string();
        let span = source_span(call.method.span());
        self.methods
            .push(self.existing.get(&(span, name.as_str())).map_or_else(
                || {
                    fact(
                        name.clone(),
                        call.method.span(),
                        AnalysisQuality::Conservative,
                    )
                },
                |existing| (*existing).clone(),
            ));
        visit::visit_expr_method_call(self, call);
    }
}
