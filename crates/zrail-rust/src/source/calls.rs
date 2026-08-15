//! Direct function and associated-function calls with conservative glob candidates.

use syn::{Expr, ExprCall, spanned::Spanned as _};
use zrail_core::AnalysisQuality;

use super::{ObservedFact, fact::fact, imports::ImportMap};

pub(super) fn facts(call: &ExprCall, imports: &ImportMap) -> Vec<ObservedFact> {
    let Expr::Path(callee) = call.func.as_ref() else {
        return Vec::new();
    };
    let (resolved, quality) = imports.resolve(&callee.path);
    if resolved.is_empty() {
        return Vec::new();
    }
    let span = callee.path.span();
    let mut observed = vec![fact(resolved.clone(), span, quality)];
    if callee.qself.is_none() {
        observed.extend(candidates(&callee.path, imports, &resolved));
    }
    observed
}

pub(super) fn candidates(
    path: &syn::Path,
    imports: &ImportMap,
    resolved: &str,
) -> Vec<ObservedFact> {
    imports
        .call_candidates(path)
        .into_iter()
        .filter(|candidate| candidate != resolved)
        .map(|candidate| fact(candidate, path.span(), AnalysisQuality::Conservative))
        .collect()
}

#[cfg(test)]
#[path = "calls_test.rs"]
mod calls_test;
