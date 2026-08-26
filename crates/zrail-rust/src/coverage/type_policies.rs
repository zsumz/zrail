//! Exact type coverage reuses the enforcement resolver and macro closure.

mod declarations;
mod duplication;
mod support;

use zrail_core::{
    CloneCopyPolicy, PolicyReachability, RustFieldContract, RustTypeContract, RustTypeKind,
    TypeProhibition,
};

use crate::{
    engine::RepositoryModel,
    rules::RuleContext,
    source::{RustFileFacts, TypeDeclarationFact},
};

use super::{GovernedTypeField, GovernedTypePolicy};

pub(super) struct SelectedDeclaration<'a> {
    pub(super) file: &'a RustFileFacts,
    pub(super) declaration: &'a TypeDeclarationFact,
}

pub(super) fn report(model: &RepositoryModel) -> Vec<GovernedTypePolicy> {
    let context = context(model);
    let mut policies = model
        .bundle
        .contract
        .source
        .rust
        .types
        .iter()
        .map(|policy| policy_report(model, &context, policy))
        .collect::<Vec<_>>();
    policies.sort_by(|left, right| left.policy_id.cmp(&right.policy_id));
    policies
}

fn policy_report(
    model: &RepositoryModel,
    context: &RuleContext<'_>,
    policy: &RustTypeContract,
) -> GovernedTypePolicy {
    let (mut observations, declarations) = declarations::observations(model, context, policy);
    observations.extend(duplication::observations(
        model,
        context,
        policy,
        &declarations,
    ));
    observations.sort();
    let mut deny = policy
        .deny
        .iter()
        .map(|value| prohibition_name(*value).into())
        .collect::<Vec<_>>();
    deny.sort();
    GovernedTypePolicy {
        policy_id: format!("rust:type-policy:{}", policy.name),
        name: policy.name.clone(),
        identity: policy.identity.clone(),
        path: policy.path.clone(),
        kind: kind_name(policy.kind).into(),
        reachability: reachability_name(policy.reachability).into(),
        clone_copy: clone_copy_name(policy.clone_copy).into(),
        deny,
        visibility: policy.visibility.clone(),
        leaf_module: policy.leaf_module,
        fields: policy
            .fields
            .as_ref()
            .map(|fields| fields.iter().map(expected_field).collect()),
        reason: policy.reason.clone(),
        observations,
    }
}

fn context(model: &RepositoryModel) -> RuleContext<'_> {
    RuleContext {
        contract: &model.bundle.contract,
        lock: None,
        inventory: &model.inventory,
        cargo: &model.cargo,
        resolved_cargo: model.resolved_cargo.as_ref(),
        source: &model.source,
        module_edges: &model.module_edges,
        compilation_domains: &model.compilation_domains,
        feature_worlds: &model.feature_worlds,
    }
}

fn expected_field(field: &RustFieldContract) -> GovernedTypeField {
    GovernedTypeField {
        name: field.name.clone(),
        type_identity: field.type_identity.clone(),
        visibility: field.visibility.clone(),
    }
}

const fn kind_name(value: RustTypeKind) -> &'static str {
    match value {
        RustTypeKind::Type => "type",
        RustTypeKind::AuthorityToken => "authority-token",
    }
}

const fn reachability_name(value: PolicyReachability) -> &'static str {
    match value {
        PolicyReachability::All => "all",
        PolicyReachability::Production => "production",
    }
}

const fn clone_copy_name(value: CloneCopyPolicy) -> &'static str {
    match value {
        CloneCopyPolicy::Allow => "allow",
        CloneCopyPolicy::Forbidden => "forbidden",
    }
}

const fn prohibition_name(value: TypeProhibition) -> &'static str {
    match value {
        TypeProhibition::DeriveClone => "derive-clone",
        TypeProhibition::DeriveCopy => "derive-copy",
        TypeProhibition::ImplClone => "impl-clone",
        TypeProhibition::ImplCopy => "impl-copy",
        TypeProhibition::OpaqueExpansion => "opaque-expansion",
    }
}

#[cfg(test)]
#[path = "type_policies_test.rs"]
mod type_policies_test;

#[cfg(test)]
pub(super) use type_policies_test::assert_type_policy_coverage;
