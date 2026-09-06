//! Canonical identities for configured Rust policy families.

use super::GovernedTestMirror;

pub(super) fn extend(
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
    if let Some(policy) = &rust.budgets {
        rails.push("rust:size:independent-hard".into());
        rails.extend(
            policy
                .overrides
                .iter()
                .map(|scope| format!("rust:size:scope:{}", scope.name)),
        );
        rails.extend(
            policy
                .exceptions
                .iter()
                .map(|exception| format!("rust:size:exception:{}", exception.path)),
        );
    }
}

const fn duplication_trait_name(value: zrail_core::DuplicationTrait) -> &'static str {
    match value {
        zrail_core::DuplicationTrait::Clone => "clone",
        zrail_core::DuplicationTrait::Copy => "copy",
    }
}
