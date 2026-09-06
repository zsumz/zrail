//! Every configured rail receives one stable audit identity.

use crate::engine::RepositoryModel;

mod rust;

use super::{GovernedDependencyRule, GovernedOwnerRule, GovernedTestMirror};

pub(super) fn report(
    model: &RepositoryModel,
    owners: &[GovernedOwnerRule],
    dependencies: &[GovernedDependencyRule],
    mirrors: &[GovernedTestMirror],
) -> Vec<String> {
    let contract = &model.bundle.contract;
    let rust = &contract.source.rust;
    let mut rails = vec![
        "repository:workspace-members".into(),
        "repository:nested-git".into(),
        "repository:submodules".into(),
        "repository:symlinks".into(),
        "dependencies:mode".into(),
        "dependencies:unassigned-packages".into(),
        "dependencies:cycles".into(),
        "rust:module-docs".into(),
        "rust:facades".into(),
        "rust:entrypoints".into(),
        "rust:tests".into(),
        "rust:macro-expansion".into(),
        "rust:hygiene:unsafe".into(),
        "rust:hygiene:lint-suppressions".into(),
        analysis_limit(
            "derived-source-instances",
            contract.analysis.limits.derived_source_instances,
        ),
        analysis_limit(
            "include-projection-work",
            contract.analysis.limits.include_projection_work,
        ),
        analysis_limit("projected-facts", contract.analysis.limits.projected_facts),
    ];
    rails.extend(
        contract
            .adapters
            .iter()
            .map(|adapter| format!("adapter:{adapter}")),
    );
    rails.extend(
        contract
            .repository
            .roots
            .iter()
            .map(|root| format!("repository:root:{root}")),
    );
    rails.extend(
        contract
            .repository
            .exclude
            .iter()
            .map(|path| format!("repository:exclude:{path}")),
    );
    rails.extend(
        model
            .bundle
            .sources
            .iter()
            .map(|source| format!("contract-source:{}", source.path)),
    );
    rails.extend(contract.dependencies.crate_roots.iter().map(|source| {
        format!(
            "dependency:crate-root:{}:{}",
            source.package,
            source.source.identity()
        )
    }));
    rails.extend(
        contract
            .profiles
            .keys()
            .map(|name| format!("profile:{name}")),
    );
    rails.extend(contract.profiles.iter().flat_map(|(name, profile)| {
        profile.syntax.deny.iter().map(move |syntax| {
            format!(
                "profile:{name}:syntax:{}",
                crate::rules::async_syntax_name(*syntax)
            )
        })
    }));
    rails.extend(
        contract
            .layers
            .iter()
            .map(|layer| format!("layer:{}", layer.name)),
    );
    rails.extend(dependencies.iter().map(|rule| rule.policy_id.clone()));
    rails.extend(
        contract
            .scopes
            .iter()
            .map(|scope| format!("scope:{}", scope.name)),
    );
    rails.extend(owners.iter().map(|owner| owner.policy_id.clone()));
    rails.extend(contract.ratchets.iter().map(|ratchet| {
        format!(
            "ratchet:{}:{}:{}",
            ratchet.rule,
            ratchet.selector.as_deref().unwrap_or("all"),
            ratchet.target
        )
    }));
    rails.extend(
        contract
            .gates
            .iter()
            .map(|gate| format!("gate:{}", gate.name)),
    );
    rails.extend(
        contract
            .invariants
            .iter()
            .map(|invariant| format!("invariant:{}", invariant.id)),
    );
    rust::extend(rust, mirrors, &mut rails);
    rails.extend(
        model
            .repository_files
            .policies
            .iter()
            .map(|policy| policy.policy_id.clone()),
    );
    rails.sort();
    rails.dedup();
    rails
}

fn analysis_limit(name: &str, value: Option<usize>) -> String {
    format!(
        "analysis:limit:{name}:{}",
        value.map_or_else(|| "input-derived".into(), |value| value.to_string())
    )
}
