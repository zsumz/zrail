//! Non-convergent feature worlds identify one exact Cargo edge witness.

use std::collections::BTreeSet;

use super::{FeatureClosure, FeatureWorldSpec};
use crate::cargo::{CargoWorkspace, Dependency};

pub(super) fn nonempty_split_context(
    spec: &FeatureWorldSpec,
    upper: &FeatureClosure,
    split_contexts: &super::contexts::SplitContexts,
) -> Option<String> {
    split_contexts.packages().find_map(|package| {
        let feature = upper.resolved.get(package)?.active.first()?;
        let witness = split_contexts.witness(package)?;
        Some(format!(
            "feature world {:?} is not exact: context-split package {package:?} has active feature {feature:?}; {witness}; without a Cargo compilation-unit graph, zrail accepts a context-split package only when its upper active feature set is empty",
            spec.name
        ))
    })
}

pub(super) fn non_convergent_world(
    cargo: &CargoWorkspace,
    spec: &FeatureWorldSpec,
    lower: &FeatureClosure,
    upper: &FeatureClosure,
    split_contexts: &super::contexts::SplitContexts,
) -> String {
    let detail = divergence_witness(cargo, lower, upper, split_contexts).unwrap_or_else(|| {
        "the lower and upper feature closures differ without an attributable edge witness"
            .to_owned()
    });
    format!(
        "feature world {:?} is not exact: {detail}; exact feature worlds require every relevant Cargo compilation context to converge on one feature set per package",
        spec.name
    )
}

fn divergence_witness(
    cargo: &CargoWorkspace,
    lower: &FeatureClosure,
    upper: &FeatureClosure,
    split_contexts: &super::contexts::SplitContexts,
) -> Option<String> {
    for source in &cargo.packages {
        let resolution = upper.resolved.get(&source.name)?;
        for dependency in &source.dependencies {
            let Some(destination) = dependency.internal_package() else {
                continue;
            };
            if !split_contexts.edge_is_split(source, dependency)
                || dependency.optional
                    && !resolution.enabled_dependencies.contains(&dependency.alias)
            {
                continue;
            }
            for feature in requested_features(cargo, destination, dependency, resolution) {
                if is_upper_only(destination, &feature, lower, upper) {
                    let context = split_contexts
                        .source_context(source)
                        .map_or_else(String::new, |why| format!("; {why}"));
                    return Some(format!(
                        "package {destination:?} feature {feature:?} is activated only by the {} dependency edge from package {:?} (alias {:?}, target condition {}){context}",
                        super::contexts::kind_name(dependency.kind),
                        source.name,
                        dependency.alias,
                        super::contexts::target_name(dependency.target.as_deref())
                    ));
                }
            }
        }
    }
    upper_only_feature(lower, upper).map(|(package, feature)| {
        format!(
            "package {package:?} feature {feature:?} differs between the lower closure that excludes target-conditional, build, development, and proc-macro host edges and the upper closure that includes them"
        )
    })
}

fn requested_features(
    cargo: &CargoWorkspace,
    destination: &str,
    dependency: &Dependency,
    source: &crate::cargo::PackageFeatureResolution,
) -> BTreeSet<String> {
    let mut features = dependency.features.iter().cloned().collect::<BTreeSet<_>>();
    if dependency.default_features
        && cargo.package_features[destination]
            .declared()
            .contains("default")
    {
        features.insert("default".into());
    }
    features.extend(
        source
            .dependency_features
            .get(&dependency.alias)
            .into_iter()
            .flatten()
            .cloned(),
    );
    features
}

fn is_upper_only(
    package: &str,
    feature: &str,
    lower: &FeatureClosure,
    upper: &FeatureClosure,
) -> bool {
    !lower.requested[package].contains(feature)
        && !lower.resolved[package].active.contains(feature)
        && upper.requested[package].contains(feature)
        && upper.resolved[package].active.contains(feature)
}

fn upper_only_feature(lower: &FeatureClosure, upper: &FeatureClosure) -> Option<(String, String)> {
    upper.resolved.iter().find_map(|(package, resolution)| {
        resolution
            .active
            .difference(&lower.resolved[package].active)
            .next()
            .map(|feature| (package.clone(), feature.clone()))
    })
}
