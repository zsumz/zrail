//! Projection runs only when written facts can reach a nontrivial Rust namespace.

use std::collections::BTreeSet;

use zrail_core::AnalysisQuality;

use super::IncludeBindings;
use crate::source::{ImplicitPreludeEligibility, RustFileFacts};

impl IncludeBindings {
    pub(in crate::source) fn requires_ordinary_resolution(&self, file: &RustFileFacts) -> bool {
        if file.paths.iter().chain(&file.calls).any(|fact| {
            fact.quality == AnalysisQuality::Unresolved
                && fact
                    .written
                    .as_deref()
                    .is_some_and(|written| !written.trim_start_matches("::").contains("::"))
        }) {
            return true;
        }
        if file
            .paths
            .iter()
            .chain(&file.calls)
            .any(|fact| match fact.implicit_prelude {
                ImplicitPreludeEligibility::LocalShadow => fact
                    .written
                    .as_deref()
                    .is_some_and(|written| fact.name != written),
                ImplicitPreludeEligibility::GenericShadow
                | ImplicitPreludeEligibility::PossibleShadow => fact.written.is_some(),
                ImplicitPreludeEligibility::Eligible | ImplicitPreludeEligibility::Disabled => {
                    false
                }
            })
        {
            return true;
        }
        if file.paths.iter().chain(&file.calls).any(|fact| {
            matches!(
                fact.implicit_prelude,
                ImplicitPreludeEligibility::Eligible | ImplicitPreludeEligibility::GenericShadow
            ) && fact.written.as_deref().is_some_and(|written| {
                let Some(root) = written_root(written) else {
                    return false;
                };
                self.instances.for_file(&file.relative).iter().any(|id| {
                    self.instances.get(*id).is_some_and(|source| {
                        super::implicit_prelude_catalog::core(root, &source.domain.edition)
                            .is_some()
                            || super::implicit_prelude_catalog::std_only(root).is_some()
                    })
                })
            })
        }) {
            return true;
        }
        let roots = file
            .paths
            .iter()
            .chain(&file.calls)
            .filter(|fact| {
                matches!(
                    fact.implicit_prelude,
                    ImplicitPreludeEligibility::Eligible | ImplicitPreludeEligibility::Disabled
                )
            })
            .filter_map(|fact| fact.written.as_deref())
            .filter_map(written_root)
            .collect::<BTreeSet<_>>();
        self.instances.for_file(&file.relative).iter().any(|id| {
            roots
                .iter()
                .any(|root| is_qualifier(root) || self.ancestor_can_bind(*id, root))
        })
    }

    fn ancestor_can_bind(&self, mut id: crate::source::SourceInstanceId, root: &str) -> bool {
        loop {
            let Some(instance) = self.instances.get(id) else {
                return false;
            };
            if self.files.get(&instance.file).is_some_and(|bindings| {
                bindings.named.contains_key(root) || !bindings.globs.is_empty()
            }) || self
                .opaque_namespace_scopes
                .get(&instance.file)
                .is_some_and(|scopes| !scopes.is_empty())
            {
                return true;
            }
            let Some(parent) = instance.parent else {
                return false;
            };
            id = parent;
        }
    }
}

fn written_root(path: &str) -> Option<&str> {
    let root = path.trim_start_matches("::").split("::").next()?;
    let root = root.strip_prefix("r#").unwrap_or(root);
    (!root.is_empty()).then_some(root)
}

fn is_qualifier(root: &str) -> bool {
    root.starts_with('<') || matches!(root, "crate" | "self" | "super" | "Self")
}
