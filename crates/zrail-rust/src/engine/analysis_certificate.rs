//! Candidate locks bind the exact complete input universe and work census.

use zrail_core::{LOCK_SEMANTICS, LockedAnalysis, LockedContractSource, sha256_hex};

use super::model::RepositoryModel;

pub(super) fn locked(model: &RepositoryModel) -> LockedAnalysis {
    let mut inventory = Vec::new();
    let mut packages = model.cargo.packages.iter().collect::<Vec<_>>();
    packages.sort_by(|left, right| left.name.cmp(&right.name));
    for package in &packages {
        record(&mut inventory, "package", &package.name);
        record(&mut inventory, "manifest", &package.manifest_path());
        for definition in model.cargo.package_features[&package.name].canonical_definition() {
            record(
                &mut inventory,
                &format!("package-feature-definition:{}", package.name),
                &definition,
            );
        }
        let mut targets = package.targets.iter().collect::<Vec<_>>();
        targets.sort();
        for target in targets {
            record(&mut inventory, "target-package", &package.name);
            record(&mut inventory, "target-name", &target.name);
            record(
                &mut inventory,
                "target-path",
                &package_path(&package.directory, &target.path),
            );
            record(&mut inventory, "target-kind", target_kind(target.kind));
            for required in &target.required_features {
                record(&mut inventory, "target-required-feature", required);
            }
        }
    }
    let mut feature_worlds = model.feature_worlds.iter().collect::<Vec<_>>();
    feature_worlds.sort_by(|left, right| left.name.cmp(&right.name));
    for world in feature_worlds {
        record(&mut inventory, "feature-world", &world.name);
        for (package, selection) in &world.packages {
            let prefix = format!("feature-world:{}:{package}", world.name);
            record(
                &mut inventory,
                &format!("{prefix}:default-features"),
                if selection.default_features {
                    "true"
                } else {
                    "false"
                },
            );
            for feature in &selection.selected {
                record(&mut inventory, &format!("{prefix}:selected"), feature);
            }
            for feature in &selection.active {
                record(&mut inventory, &format!("{prefix}:active"), feature);
            }
        }
    }
    for file in &model.source.files {
        record(&mut inventory, "rust", &file.relative);
    }
    let cargo_lock_sha256 = model
        .resolved_cargo
        .as_ref()
        .map(|graph| graph.lock_sha256().to_owned());
    if let Some(digest) = &cargo_lock_sha256 {
        record(&mut inventory, "cargo-lock", digest);
    }
    let mut exclusion_bytes = Vec::new();
    let mut exclusions = model
        .bundle
        .contract
        .repository
        .exclude
        .iter()
        .collect::<Vec<_>>();
    exclusions.sort();
    for exclusion in exclusions {
        record(&mut exclusion_bytes, "exclude", exclusion);
    }
    let metrics = model.source.analysis_metrics;
    LockedAnalysis {
        inventory_sha256: sha256_hex(&inventory),
        exclusions_sha256: sha256_hex(&exclusion_bytes),
        cargo_lock_sha256,
        cargo_features_sha256: cargo_features_sha256(model),
        feature_worlds_sha256: feature_worlds_sha256(model),
        feature_worlds: Some(model.feature_worlds.len()),
        packages: packages.len(),
        targets: packages.iter().map(|package| package.targets.len()).sum(),
        physical_rust_files: model.source.files.len(),
        base_source_contexts: metrics.base_contexts,
        derived_source_contexts: metrics.derived_contexts,
        source_facts: model
            .source
            .files
            .iter()
            .map(crate::source::fact_count)
            .sum(),
        projection_queries: metrics.projection_work,
        projected_facts: metrics.projected_facts,
        unresolved_bindings: 0,
        analyzer_semantics: LOCK_SEMANTICS,
        contract_sources: model
            .bundle
            .sources
            .iter()
            .map(|source| LockedContractSource {
                path: source.path.clone(),
                sha256: sha256_hex(source.content.as_bytes()),
            })
            .collect(),
    }
}

fn cargo_features_sha256(model: &RepositoryModel) -> String {
    let mut authority = Vec::new();
    for (package, features) in &model.cargo.package_features {
        record(&mut authority, "package", package);
        for definition in features.canonical_definition() {
            record(&mut authority, "definition", &definition);
        }
        if let Some(package) = model
            .cargo
            .packages
            .iter()
            .find(|candidate| candidate.name == *package)
        {
            for target in &package.targets {
                for required in &target.required_features {
                    record(
                        &mut authority,
                        &format!("target-required-feature:{}", target.name),
                        required,
                    );
                }
            }
        }
    }
    sha256_hex(&authority)
}

fn feature_worlds_sha256(model: &RepositoryModel) -> String {
    let mut authority = Vec::new();
    let mut worlds = model.feature_worlds.iter().collect::<Vec<_>>();
    worlds.sort_by(|left, right| left.name.cmp(&right.name));
    for world in worlds {
        record(&mut authority, "world", &world.name);
        for (package, selection) in &world.packages {
            record(&mut authority, "package", package);
            record(
                &mut authority,
                "default-features",
                if selection.default_features {
                    "true"
                } else {
                    "false"
                },
            );
            for feature in &selection.selected {
                record(&mut authority, "selected", feature);
            }
            for feature in &selection.active {
                record(&mut authority, "active", feature);
            }
        }
    }
    sha256_hex(&authority)
}

fn record(output: &mut Vec<u8>, label: &str, value: &str) {
    output.extend_from_slice(label.len().to_string().as_bytes());
    output.push(b':');
    output.extend_from_slice(label.as_bytes());
    output.push(0);
    output.extend_from_slice(value.len().to_string().as_bytes());
    output.push(b':');
    output.extend_from_slice(value.as_bytes());
    output.push(0xff);
}

fn package_path(directory: &str, path: &str) -> String {
    if directory == "." {
        path.into()
    } else {
        format!("{directory}/{path}")
    }
}

const fn target_kind(kind: crate::cargo::CargoTargetKind) -> &'static str {
    use crate::cargo::CargoTargetKind;
    match kind {
        CargoTargetKind::Library => "library",
        CargoTargetKind::Binary => "binary",
        CargoTargetKind::Example => "example",
        CargoTargetKind::Test => "test",
        CargoTargetKind::Benchmark => "benchmark",
        CargoTargetKind::BuildScript => "build-script",
    }
}
