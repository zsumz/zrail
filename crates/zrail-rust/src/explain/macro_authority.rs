//! Path guidance names every observed content-bound repository macro package.

use std::collections::BTreeSet;

use crate::{engine::RepositoryModel, source::MacroOrigin};

pub(super) fn implementations(model: &RepositoryModel) -> Vec<String> {
    let allowed = model
        .bundle
        .contract
        .source
        .rust
        .macros
        .allow
        .iter()
        .map(|allowance| allowance.name.as_str())
        .collect::<BTreeSet<_>>();
    model
        .source
        .files
        .iter()
        .flat_map(|file| &file.macro_expansions)
        .flat_map(|expansion| {
            expansion
                .policy_names()
                .filter(|name| allowed.contains(name))
                .flat_map(|name| {
                    expansion
                        .origins
                        .iter()
                        .filter_map(move |origin| match origin {
                            MacroOrigin::Repository { package, directory } => {
                                Some(format!("{name}@{package}:{directory}"))
                            }
                            _ => None,
                        })
                })
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
