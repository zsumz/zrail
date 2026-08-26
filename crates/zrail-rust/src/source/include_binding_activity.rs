//! Active compilation instances are indexed once per file and cfg guard.

use std::collections::{BTreeMap, BTreeSet};

use super::{SourceIndex, SourceInstanceId, SourceInstances, SyntaxGuard};

pub(super) fn active_instances(
    index: &SourceIndex,
    instances: &SourceInstances,
) -> BTreeMap<(String, SyntaxGuard), Vec<SourceInstanceId>> {
    index
        .files
        .iter()
        .flat_map(|file| {
            file.paths
                .iter()
                .chain(&file.calls)
                .filter(|fact| fact.written.is_some())
                .map(|fact| (file.relative.clone(), fact.guard.clone()))
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|(file, guard)| {
            let active = instances
                .for_file(&file)
                .iter()
                .copied()
                .filter(|id| {
                    instances.get(*id).is_some_and(|instance| {
                        guard
                            .availability_in_domain(&instance.domain)
                            .is_available()
                    })
                })
                .collect();
            ((file, guard), active)
        })
        .collect()
}
