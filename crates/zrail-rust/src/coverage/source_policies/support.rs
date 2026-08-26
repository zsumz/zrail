//! Shared source-policy occurrence rendering and deterministic vocabulary.

use zrail_core::{DuplicationTrait, GlobImportMode, PolicyReachability};

use crate::{
    engine::RepositoryModel,
    rules::async_syntax_name,
    source::{BindingVisibility, CompilationMode, ObservedFact, RustFileFacts, SyntaxGuard},
};

use super::super::{GovernedCompilationDomain, GovernedSourcePolicyOccurrence};

pub(super) fn direct_occurrence(
    model: &RepositoryModel,
    file: &RustFileFacts,
    fact: &crate::source::AsyncSyntaxFact,
) -> Option<GovernedSourcePolicyOccurrence> {
    Some(GovernedSourcePolicyOccurrence {
        path: file.relative.clone(),
        operation: async_syntax_name(fact.kind).into(),
        observed: fact.observation.name.clone(),
        visibility: None,
        lexical_scope: fact.observation.lexical_scope.clone(),
        span: fact.observation.span?,
        quality: fact.observation.quality,
        guard: fact.observation.guard.canonical_name(),
        compilation_domains: domains(model, &file.relative, &fact.observation.guard),
        allowed: false,
    })
}

pub(super) fn macro_occurrence(
    model: &RepositoryModel,
    file: &RustFileFacts,
    expansion: &crate::source::MacroExpansionFact,
    allowed: bool,
) -> Option<GovernedSourcePolicyOccurrence> {
    Some(GovernedSourcePolicyOccurrence {
        path: file.relative.clone(),
        operation: "macro-expansion".into(),
        observed: expansion.name.clone(),
        visibility: None,
        lexical_scope: expansion.lexical_scope.clone(),
        span: expansion.span?,
        quality: expansion.quality,
        guard: expansion.guard.canonical_name(),
        compilation_domains: domains(model, &file.relative, &expansion.guard),
        allowed,
    })
}

pub(super) fn applies(
    policy: PolicyReachability,
    file: &RustFileFacts,
    fact: &ObservedFact,
) -> bool {
    fact.guard != SyntaxGuard::Never
        && (policy == PolicyReachability::All || fact.is_production_applicable(file.reachability))
}

pub(super) fn domains(
    model: &RepositoryModel,
    path: &str,
    guard: &SyntaxGuard,
) -> Vec<GovernedCompilationDomain> {
    model
        .compilation_domains
        .get(path)
        .into_iter()
        .flatten()
        .filter(|domain| guard.availability_in_domain(domain).is_available())
        .map(|domain| GovernedCompilationDomain {
            package: domain.package.clone(),
            edition: domain.edition.clone(),
            target: domain.target.clone(),
            mode: compilation_mode(domain.mode).into(),
            feature_world: domain.feature_world.clone(),
            features: domain.active_features.iter().cloned().collect(),
        })
        .collect()
}

pub(super) fn policy_domains(
    model: &RepositoryModel,
    path: &str,
    guard: &SyntaxGuard,
    reachability: PolicyReachability,
) -> Vec<GovernedCompilationDomain> {
    domains(model, path, guard)
        .into_iter()
        .filter(|domain| {
            reachability == PolicyReachability::All
                || matches!(domain.mode.as_str(), "library" | "binary")
        })
        .collect()
}

pub(super) fn visibility(value: &BindingVisibility) -> String {
    match value {
        BindingVisibility::Public => "public".into(),
        BindingVisibility::Private => "private".into(),
        BindingVisibility::Restricted(path) => format!("restricted:{}", path.join("::")),
    }
}

pub(super) const fn glob_mode(value: GlobImportMode) -> &'static str {
    match value {
        GlobImportMode::Allow => "allow",
        GlobImportMode::FacadeReexportsOnly => "facade-reexports-only",
        GlobImportMode::Deny => "deny",
    }
}

pub(super) const fn reachability(value: PolicyReachability) -> &'static str {
    match value {
        PolicyReachability::All => "all",
        PolicyReachability::Production => "production",
    }
}

pub(super) const fn duplication_trait_name(value: DuplicationTrait) -> &'static str {
    match value {
        DuplicationTrait::Clone => "clone",
        DuplicationTrait::Copy => "copy",
    }
}

const fn compilation_mode(value: CompilationMode) -> &'static str {
    match value {
        CompilationMode::Library => "library",
        CompilationMode::LibraryTest => "library-test",
        CompilationMode::Binary => "binary",
        CompilationMode::BinaryTest => "binary-test",
        CompilationMode::IntegrationTest => "integration-test",
        CompilationMode::Benchmark => "benchmark",
        CompilationMode::Example => "example",
        CompilationMode::ExampleTest => "example-test",
        CompilationMode::BuildScript => "build-script",
    }
}
