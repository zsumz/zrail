//! Git object kinds remain authoritative when a filesystem snapshot cannot represent them.

use zrail_core::PolicyMode;

use crate::app::error::CliError;

use super::super::git_base::GitSnapshot;

pub(in crate::app::commands) fn require_submodule_policy(
    snapshot: &GitSnapshot,
    policy: PolicyMode,
) -> Result<(), CliError> {
    if policy == PolicyMode::Deny
        && let Some((path, entry)) = snapshot.tree.iter().find(|(_, entry)| entry.is_gitlink())
    {
        return Err(CliError::new(format!(
            "migration target Git gitlink {path:?} (mode {}, object {}) is denied by repository.submodules",
            entry.mode, entry.object
        )));
    }
    // Allowed gitlinks are not materialized or traversed. The report retains
    // their mode and object digest, and its target commit binds unchanged links.
    Ok(())
}
