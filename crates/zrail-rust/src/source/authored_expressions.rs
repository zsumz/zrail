//! Authored path membership retains written syntax without changing invocation authority.
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
    collect_membership(syntax, paths, calls, false)
}

pub(super) fn collect_all(
    syntax: &syn::File,
    paths: &[ObservedFact],
    calls: &[ObservedFact],
) -> Vec<ObservedFact> {
    collect_membership(syntax, paths, calls, true)
}

fn collect_membership(
    syntax: &syn::File,
    paths: &[ObservedFact],
    calls: &[ObservedFact],
    all_paths: bool,
) -> Vec<ObservedFact> {
    let existing = paths
        .iter()
        .chain(calls)
        .filter_map(|fact| Some(((fact.span?, fact.written.as_deref()?), fact)))
        .collect();
    let mut visitor = AuthoredPaths {
        existing,
        paths: Vec::new(),
        all_paths,
    };
    visitor.visit_file(syntax);
    visitor.paths
}

struct AuthoredPaths<'a> {
    existing: BTreeMap<(SourceSpan, &'a str), &'a ObservedFact>,
    paths: Vec<ObservedFact>,
    all_paths: bool,
}

impl AuthoredPaths<'_> {
    fn record(&mut self, path: &syn::Path) -> bool {
        // Preserve one excess observation for the existing parser fact limit.
        if self.paths.len() > super::MAX_FACTS_PER_FILE {
            return false;
        }
        let written = written_path(path);
        let span = path.span();
        self.paths.push(
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
        true
    }
}

impl<'ast> Visit<'ast> for AuthoredPaths<'_> {
    fn visit_expr_path(&mut self, expression: &'ast syn::ExprPath) {
        if !self.all_paths && !self.record(&expression.path) {
            return;
        }
        visit::visit_expr_path(self, expression);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        if self.all_paths && !self.record(path) {
            return;
        }
        visit::visit_path(self, path);
    }
}
