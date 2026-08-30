//! Git ancestry, file manifests, and worktree identity for migration bridges.

mod manifest;
mod topology;
mod worktree;

pub(super) use manifest::{changes, require_report_output};
pub(super) use topology::require_submodule_policy;
pub(super) use worktree::{require_ancestor, require_worktree_target};
