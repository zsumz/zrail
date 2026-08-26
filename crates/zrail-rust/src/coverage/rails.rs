//! Every configured rail receives one stable audit identity.

use crate::engine::RepositoryModel;

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
    rust_rails(rust, mirrors, &mut rails);
    rails.sort();
    rails.dedup();
    rails
}

fn rust_rails(
    rust: &zrail_core::RustSourceContract,
    mirrors: &[GovernedTestMirror],
    rails: &mut Vec<String>,
) {
    rails.extend(
        rust.file_roles
            .iter()
            .map(|role| format!("rust:file-role:{}", role.path)),
    );
    rails.extend(
        rust.generated
            .iter()
            .map(|source| format!("rust:generated:{}", source.root)),
    );
    rails.extend(
        rust.out_dir
            .iter()
            .map(|source| format!("rust:out-dir:{}:{}", source.path, source.output)),
    );
    rails.extend(rust.item_macros.iter().map(|item| {
        format!(
            "rust:item-macro:{}:{}",
            item.name,
            item.path
                .as_deref()
                .map_or_else(|| item.within.join(","), str::to_owned)
        )
    }));
    rails.extend(mirrors.iter().map(|mirror| mirror.policy_id.clone()));
    rails.push(if rust.feature_worlds.is_empty() {
        "rust:feature-world-mode:legacy-conditional".into()
    } else {
        "rust:feature-world-mode:exact".into()
    });
    rails.extend(
        rust.feature_worlds
            .iter()
            .map(|world| format!("rust:feature-world:{}", world.name)),
    );
    rails.extend(
        rust.macros
            .allow
            .iter()
            .map(|allowance| format!("rust:macro:{}", allowance.name)),
    );
    rails.extend(
        rust.duplication
            .deny_imports
            .iter()
            .map(|value| format!("rust:duplication:import:{}", duplication_trait_name(*value))),
    );
    rails.extend(rust.duplication.deny_macro_tokens.iter().map(|value| {
        format!(
            "rust:duplication:macro-token:{}",
            duplication_trait_name(*value)
        )
    }));
    rails.extend(
        rust.types
            .iter()
            .map(|policy| format!("rust:type-policy:{}", policy.name)),
    );
    rails.extend(
        rust.hygiene
            .deny_methods
            .iter()
            .map(|method| format!("rust:hygiene:denied-method:{method}")),
    );
    rails.push("rust:hygiene:glob-imports".into());
    rails.extend(
        rust.hygiene
            .deny_macros
            .iter()
            .map(|name| format!("rust:hygiene:denied-macro:{name}")),
    );
    if rust.size.is_some() {
        rails.extend(
            ["facade", "implementation", "test", "auxiliary"]
                .map(|role| format!("rust:size:{role}")),
        );
    }
}

const fn duplication_trait_name(value: zrail_core::DuplicationTrait) -> &'static str {
    match value {
        zrail_core::DuplicationTrait::Clone => "clone",
        zrail_core::DuplicationTrait::Copy => "copy",
    }
}

fn analysis_limit(name: &str, value: Option<usize>) -> String {
    format!(
        "analysis:limit:{name}:{}",
        value.map_or_else(|| "input-derived".into(), |value| value.to_string())
    )
}
