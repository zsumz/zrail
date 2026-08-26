//! Owner coverage reuses the exact source-operation matcher used by enforcement.

use zrail_core::{AnalysisQuality, OwnerContract, OwnerKind, PolicyReachability, glob_matches};

use crate::{
    engine::RepositoryModel,
    rules::{
        CallOwnerEvidenceKind, matching_call_owner_evidence, matching_capability_owner,
        matching_directory_owner, matching_operation_owner_operations,
    },
    source::{
        CompilationMode, ObservedFact, RustFileFacts, SourceOperationFact, SourceOperationKind,
    },
};

use super::{GovernedCompilationDomain, GovernedOperationOccurrence, GovernedOwnerRule};

pub(super) fn report(model: &RepositoryModel) -> Vec<GovernedOwnerRule> {
    let mut owners = model
        .bundle
        .contract
        .owners
        .iter()
        .map(|owner| {
            let kind = owner_kind(owner.kind).to_owned();
            let mut occurrences = occurrences(model, owner);
            occurrences.sort();
            GovernedOwnerRule {
                policy_id: format!("owner:{kind}:{}", owner.name),
                name: owner.name.clone(),
                kind,
                target: owner.selector.clone(),
                mutating_methods: sorted(&owner.mutating_methods),
                reachability: reachability(owner.reachability).into(),
                within: sorted(&owner.within),
                allow: sorted(&owner.allow),
                reason: owner.reason.clone(),
                occurrences,
            }
        })
        .collect::<Vec<_>>();
    owners.sort_by(|left, right| left.policy_id.cmp(&right.policy_id));
    owners
}

fn occurrences(model: &RepositoryModel, owner: &OwnerContract) -> Vec<GovernedOperationOccurrence> {
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
                matching_operation_owner_operations(owner, file)
                    .map(|operation| operation_occurrence(model, owner, &file.relative, operation))
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
        canonical: sorted(&fact.canonical),
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

fn sorted(values: &[String]) -> Vec<String> {
    let mut values = values.to_vec();
    values.sort();
    values.dedup();
    values
}

const fn owner_kind(kind: OwnerKind) -> &'static str {
    match kind {
        OwnerKind::Call => "call",
        OwnerKind::Capability => "capability",
        OwnerKind::Directory => "directory",
        OwnerKind::TypeConstruction => "type-construction",
        OwnerKind::MethodName => "method-name",
        OwnerKind::FieldRead => "field-read",
        OwnerKind::FieldWrite => "field-write",
        OwnerKind::FieldMutableBorrow => "field-mutable-borrow",
        OwnerKind::FieldMutation => "field-mutation",
        OwnerKind::FieldAuthority => "field-authority",
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

const fn reachability(value: PolicyReachability) -> &'static str {
    match value {
        PolicyReachability::All => "all",
        PolicyReachability::Production => "production",
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
