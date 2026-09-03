//! Completed written resolutions are reused only within an identical logical context.

use std::collections::BTreeMap;

use super::super::super::{
    SourceInstanceId, SyntaxGuard, include_bindings::ResolvedPath,
    include_resolution_state::ResolutionUsage,
};

#[derive(Debug, Default)]
pub(super) struct Cache {
    entries: BTreeMap<Key, Vec<ResolvedPath>>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Key {
    instance: SourceInstanceId,
    written: String,
    scope: Vec<zrail_core::SourceSpan>,
    usage: ResolutionUsage,
    guard: SyntaxGuard,
}

impl Cache {
    pub(super) fn get(
        &self,
        instance: SourceInstanceId,
        written: &str,
        scope: &[zrail_core::SourceSpan],
        usage: ResolutionUsage,
        guard: &SyntaxGuard,
    ) -> Option<Vec<ResolvedPath>> {
        self.entries
            .get(&key(instance, written, scope, usage, guard))
            .cloned()
    }

    pub(super) fn insert(
        &mut self,
        instance: SourceInstanceId,
        written: &str,
        scope: &[zrail_core::SourceSpan],
        usage: ResolutionUsage,
        guard: &SyntaxGuard,
        resolved: Vec<ResolvedPath>,
    ) {
        self.entries
            .insert(key(instance, written, scope, usage, guard), resolved);
    }
}

fn key(
    instance: SourceInstanceId,
    written: &str,
    scope: &[zrail_core::SourceSpan],
    usage: ResolutionUsage,
    guard: &SyntaxGuard,
) -> Key {
    Key {
        instance,
        written: written.into(),
        scope: scope.to_vec(),
        usage,
        guard: guard.clone(),
    }
}
