//! Canonical type identities are selected only in available compilation domains.

use std::collections::BTreeSet;

use zrail_core::{AnalysisQuality, PolicyReachability, SourceSpan};

use crate::source::{
    CompilationDomain, CompilationMode, FactNamespace, GuardAvailability, RustFileFacts,
    SyntaxGuard,
};

use super::super::RuleContext;

#[derive(Debug, Default)]
pub(crate) struct IdentityResolution {
    pub(crate) exact: BTreeSet<String>,
    pub(crate) unresolved: bool,
}

impl IdentityResolution {
    pub(crate) fn unresolved() -> Self {
        Self {
            exact: BTreeSet::new(),
            unresolved: true,
        }
    }

    pub(crate) fn is_exact(&self, expected: &str) -> bool {
        !self.unresolved
            && self.exact.len() == 1
            && self
                .exact
                .iter()
                .any(|observed| identities_match(observed, expected))
    }

    pub(crate) fn contains(&self, expected: &str) -> bool {
        self.exact
            .iter()
            .any(|observed| identities_match(observed, expected))
    }

    pub(super) fn one(&self) -> Result<&str, &'static str> {
        if self.unresolved {
            return Err("identity is unresolved");
        }
        if self.exact.len() != 1 {
            return Err("identity is absent or ambiguous");
        }
        self.exact
            .iter()
            .next()
            .map(String::as_str)
            .ok_or("identity is absent or ambiguous")
    }
}

pub(crate) fn normalize(identity: &str) -> &str {
    identity.strip_prefix("crate::").unwrap_or(identity)
}

fn identities_match(observed: &str, expected: &str) -> bool {
    normalize(observed) == normalize(expected)
}

pub(crate) fn at_span(
    context: &RuleContext<'_>,
    file: &RustFileFacts,
    span: SourceSpan,
    reachability: PolicyReachability,
    namespace: Option<FactNamespace>,
) -> IdentityResolution {
    let mut resolution = IdentityResolution::default();
    for fact in file.paths.iter().filter(|fact| {
        fact.span == Some(span)
            && namespace.is_none_or(|namespace| fact.namespace == namespace)
            && applies(context, file, &fact.guard, reachability)
    }) {
        if fact.quality == AnalysisQuality::Exact {
            resolution
                .exact
                .extend(fact.policy_names().map(str::to_owned));
        } else {
            resolution.unresolved = true;
        }
    }
    resolution
}

pub(crate) fn applies(
    context: &RuleContext<'_>,
    file: &RustFileFacts,
    guard: &SyntaxGuard,
    reachability: PolicyReachability,
) -> bool {
    if file.reachability.is_unreachable() {
        return false;
    }
    context
        .compilation_domains
        .get(&file.relative)
        .is_some_and(|domains| {
            domains.iter().any(|domain| {
                within_reachability(domain, reachability)
                    && guard.availability_in_domain(domain).is_available()
            })
        })
}

pub(crate) fn at_span_in_domain(
    file: &RustFileFacts,
    span: SourceSpan,
    domain: &CompilationDomain,
    namespace: FactNamespace,
) -> IdentityResolution {
    let mut resolution = IdentityResolution::default();
    for fact in file
        .paths
        .iter()
        .filter(|fact| fact.span == Some(span) && fact.namespace == namespace)
    {
        match fact.guard.availability_in_domain(domain) {
            GuardAvailability::Absent => {}
            GuardAvailability::Exact if fact.quality == AnalysisQuality::Exact => {
                resolution
                    .exact
                    .extend(fact.policy_names().map(str::to_owned));
            }
            _ => resolution.unresolved = true,
        }
    }
    resolution
}

pub(crate) fn domain_identities(
    context: &RuleContext<'_>,
    file: &RustFileFacts,
    guard: &SyntaxGuard,
    reachability: PolicyReachability,
) -> Vec<String> {
    context
        .compilation_domains
        .get(&file.relative)
        .into_iter()
        .flatten()
        .filter(|domain| {
            within_reachability(domain, reachability)
                && guard.availability_in_domain(domain).is_available()
        })
        .map(CompilationDomain::canonical_identity)
        .collect()
}

pub(crate) fn within_reachability(
    domain: &CompilationDomain,
    reachability: PolicyReachability,
) -> bool {
    reachability == PolicyReachability::All
        || matches!(
            domain.mode,
            CompilationMode::Library | CompilationMode::Binary
        )
}
