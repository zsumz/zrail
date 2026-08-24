//! Repository containment policy in the public contract schema.

use serde::{Deserialize, Serialize};

use super::super::modes::{ExactMode, PolicyMode, SymlinkMode};

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Repository roots, exclusions, and containment policy."] pub struct RepositoryContract {
    #[doc = "Repository-relative directories included in architecture analysis."] pub roots: Vec<String>,
    #[serde(default)]
    #[doc = "Repository-relative patterns excluded from analysis."] pub exclude: Vec<String>,
    #[doc = "Required relationship between declared and discovered workspace members."] pub workspace_members: ExactMode,
    #[doc = "Policy for nested Git repositories beneath governed roots."] pub nested_git: PolicyMode,
    #[doc = "Policy for Git submodules beneath governed roots."] pub submodules: PolicyMode,
    #[doc = "Policy for symbolic links beneath governed roots."] pub symlinks: SymlinkMode,
}
