//! Authored expression membership retains written paths without changing invocation authority.
//!
//! Reuse located path/call facts before projection. Supplement syntax contexts omitted
//! by semantic traversal; neither aliases nor opaque macro tokens expand this claim.

use std::collections::BTreeMap;

use syn::{
    spanned::Spanned,
    visit::{self, Visit},
};
use zrail_core::{AnalysisQuality, SourceSpan};

use crate::source::{
    ObservedFact,
    fact::{source_span, written_fact, written_path},
};

pub(super) fn collect(
    syntax: &syn::File,
    paths: &[ObservedFact],
    calls: &[ObservedFact],
) -> Vec<ObservedFact> {
    let existing = paths
        .iter()
        .chain(calls)
        .filter_map(|fact| Some(((fact.span?, fact.written.as_deref()?), fact)))
        .collect();
    let mut visitor = AuthoredExpressions {
        existing,
        expressions: Vec::new(),
    };
    visitor.visit_file(syntax);
    visitor.expressions
}

struct AuthoredExpressions<'a> {
    existing: BTreeMap<(SourceSpan, &'a str), &'a ObservedFact>,
    expressions: Vec<ObservedFact>,
}

impl<'ast> Visit<'ast> for AuthoredExpressions<'_> {
    fn visit_expr_path(&mut self, expression: &'ast syn::ExprPath) {
        // Preserve one excess observation for the existing parser fact limit.
        if self.expressions.len() > super::MAX_FACTS_PER_FILE {
            return;
        }
        let written = written_path(&expression.path);
        let span = expression.path.span();
        self.expressions.push(
            self.existing
                .get(&(source_span(span), written.as_str()))
                .map_or_else(
                    || {
                        written_fact(
                            written.clone(),
                            written.clone(),
                            span,
                            AnalysisQuality::Conservative,
                            &[],
                        )
                    },
                    |existing| (*existing).clone(),
                ),
        );
        visit::visit_expr_path(self, expression);
    }
}
