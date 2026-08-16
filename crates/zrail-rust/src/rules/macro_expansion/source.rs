//! External expansion authority applies only to exact dependency sources in every file context.

use zrail_core::MacroExpansionAllow;

use crate::{
    cargo::{DependencySource, rust_crate_root, source_matches},
    source::RustFileFacts,
};

use super::super::RuleContext;

pub(super) fn bound(
    context: &RuleContext<'_>,
    file: &RustFileFacts,
    allowance: &MacroExpansionAllow,
) -> bool {
    if allowance.name.starts_with("local::") {
        return allowance.source.is_none();
    }
    let root = allowance.name.split("::").next().unwrap_or(&allowance.name);
    let mut external = false;
    for package in context.cargo.packages.iter().filter(|package| {
        file.packages.contains(&package.name)
            || (file.packages.is_empty() && package.contains_file(&file.relative))
    }) {
        for dependency in &package.dependencies {
            if rust_crate_root(&dependency.name) != root
                || !matches!(
                    dependency.source,
                    DependencySource::Registry { .. } | DependencySource::Git { .. }
                )
            {
                continue;
            }
            external = true;
            if !allowance
                .source
                .as_ref()
                .is_some_and(|source| source_matches(source, &dependency.source))
            {
                return false;
            }
        }
    }
    external || allowance.source.is_none()
}
