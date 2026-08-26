//! Source-operation collection for one owner coverage rule.

use zrail_core::{AnalysisQuality, OwnerContract, OwnerKind, PolicyReachability, glob_matches};

use crate::{
    engine::RepositoryModel,
    rules::{
        CallOwnerEvidenceKind, matching_call_owner_evidence, matching_capability_owner,
        matching_directory_owner, matching_operation_owner_operations,
    },
    source::{
        CompilationMode, MacroExpansionFact, ObservedFact, RustFileFacts, SourceOperationFact,
        SourceOperationKind, SyntaxGuard,
    },
};

use super::super::{GovernedCompilationDomain, GovernedOperationOccurrence};

pub(super) fn occurrences(
    model: &RepositoryModel,
    owner: &OwnerContract,
) -> Vec<GovernedOperationOccurrence> {
    match owner.kind {
        OwnerKind::Call => source_files(model, owner)
            .flat_map(|file| {
                matching_call_owner_evidence(owner, file)
                    .into_iter()
                    .map(|evidence| {
                        fact_occurrence(
                            model,
                            owner,
                            &file.relative,
                            call_operation(evidence.kind),
                            evidence.fact,
                            None,
                        )
                    })
            })
            .collect(),
        OwnerKind::Capability => source_files(model, owner)
            .flat_map(|file| {
                matching_capability_owner(owner, file)
                    .into_iter()
                    .map(|fact| {
                        fact_occurrence(model, owner, &file.relative, "capability-use", fact, None)
                    })
            })
            .collect(),
        OwnerKind::Directory => matching_directory_owner(owner, &model.inventory)
            .map(|entry| directory_occurrence(owner, &entry.relative))
            .collect(),
        OwnerKind::TypeConstruction
        | OwnerKind::MethodName
        | OwnerKind::FieldRead
        | OwnerKind::FieldWrite
        | OwnerKind::FieldMutableBorrow
        | OwnerKind::FieldMutation
        | OwnerKind::FieldAuthority => source_files(model, owner)
            .flat_map(|file| {
                let mut occurrences = matching_operation_owner_operations(owner, file)
                    .map(|operation| operation_occurrence(model, owner, &file.relative, operation))
                    .collect::<Vec<_>>();
                occurrences.extend(
                    file.macro_expansions
                        .iter()
                        .filter(|expansion| opaque_macro_applies(model, owner, file, expansion))
                        .map(|expansion| {
                            macro_operation_occurrence(model, owner, &file.relative, expansion)
                        }),
                );
                occurrences
            })
            .collect(),
    }
}

fn source_files<'a>(
    model: &'a RepositoryModel,
    owner: &'a OwnerContract,
) -> impl Iterator<Item = &'a RustFileFacts> + 'a {
    model.source.files.iter().filter(|file| {
        owner
            .within
            .iter()
            .any(|pattern| glob_matches(pattern, &file.relative))
    })
}

fn operation_occurrence(
    model: &RepositoryModel,
    owner: &OwnerContract,
    path: &str,
    operation: &SourceOperationFact,
) -> GovernedOperationOccurrence {
    fact_occurrence(
        model,
        owner,
        path,
        operation_kind(operation.kind),
        &operation.identity,
        operation.method.clone(),
    )
}

fn opaque_macro_applies(
    model: &RepositoryModel,
    owner: &OwnerContract,
    file: &RustFileFacts,
    expansion: &MacroExpansionFact,
) -> bool {
    expansion.observation.guard != SyntaxGuard::Never
        && (owner.reachability == PolicyReachability::All
            || expansion
                .observation
                .is_production_applicable(file.reachability))
        && !crate::rules::closes_source_operations(
            &model.bundle.contract,
            &model.source,
            model.resolved_cargo.as_ref(),
            expansion,
        )
}

fn macro_operation_occurrence(
    model: &RepositoryModel,
    owner: &OwnerContract,
    path: &str,
    expansion: &MacroExpansionFact,
) -> GovernedOperationOccurrence {
    let mut candidate = expansion.observation.clone();
    candidate.name.clone_from(&owner.selector);
    candidate.written = Some(expansion.name.clone());
    candidate.canonical.clear();
    candidate.quality = AnalysisQuality::Unresolved;
    fact_occurrence(
        model,
        owner,
        path,
        "opaque-macro-source-operation",
        &candidate,
        None,
    )
}

fn fact_occurrence(
    model: &RepositoryModel,
    owner: &OwnerContract,
    path: &str,
    operation: &str,
    fact: &ObservedFact,
    method: Option<String>,
) -> GovernedOperationOccurrence {
    let domains = model
        .compilation_domains
        .get(path)
        .into_iter()
        .flatten()
        .filter(|domain| fact.guard.availability_in_domain(domain).is_available())
        .map(|domain| GovernedCompilationDomain {
            package: domain.package.clone(),
            edition: domain.edition.clone(),
            target: domain.target.clone(),
            mode: compilation_mode(domain.mode).into(),
            feature_world: domain.feature_world.clone(),
            features: domain.active_features.iter().cloned().collect(),
        })
        .collect();
    GovernedOperationOccurrence {
        path: path.into(),
        operation: operation.into(),
        observed: fact.name.clone(),
        written: fact.written.clone(),
        method,
        canonical: super::sorted(&fact.canonical),
        span: fact.span,
        quality: fact.quality,
        guard: fact.guard.canonical_name(),
        compilation_domains: domains,
        allowed: owner.allow.iter().any(|allowed| allowed == path),
    }
}

fn directory_occurrence(owner: &OwnerContract, path: &str) -> GovernedOperationOccurrence {
    GovernedOperationOccurrence {
        path: path.into(),
        operation: "directory".into(),
        observed: path.into(),
        written: None,
        method: None,
        canonical: Vec::new(),
        span: None,
        quality: AnalysisQuality::Exact,
        guard: "ordinary".into(),
        compilation_domains: Vec::new(),
        allowed: owner.allow.iter().any(|allowed| allowed == path),
    }
}

const fn call_operation(kind: CallOwnerEvidenceKind) -> &'static str {
    match kind {
        CallOwnerEvidenceKind::DirectCall => "direct-call",
        CallOwnerEvidenceKind::Reference => "reference",
    }
}

const fn operation_kind(kind: SourceOperationKind) -> &'static str {
    match kind {
        SourceOperationKind::TypeConstruction => "type-construction",
        SourceOperationKind::MethodCall => "method-call",
        SourceOperationKind::FieldReceiverCall => "field-receiver-call",
        SourceOperationKind::FieldRead => "field-read",
        SourceOperationKind::FieldWrite => "field-write",
        SourceOperationKind::FieldMutableBorrow => "field-mutable-borrow",
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
