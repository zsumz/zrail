//! Path guidance preserves complete scope totals without copying unrelated file observations.

use serde::{Deserialize, Serialize};
use zrail_core::{RepositoryFilePredicate, RepositoryFileRule, glob_matches};

use crate::{GovernedRepositoryFileEntry, engine::RepositoryModel};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Actual observations and effective repository-file policy for one explained path.
pub struct RepositoryFileExplanation {
    /// Canonical policy identity.
    pub policy_id: String,
    /// Full authored predicate, selector, exclusions, and justification.
    pub policy: RepositoryFileRule,
    /// Physical-path, raw-UTF-8-text, or file-byte claim; never execution evidence.
    pub claim: String,
    /// Bound canonical checkout prefix when filesystem name semantics were selected.
    pub checkout_path: Option<String>,
    /// Actual selected path observation; absent for hypothetical paths or other entry kinds.
    pub entry: Option<GovernedRepositoryFileEntry>,
    /// Whether this path is the byte-equality reference rather than only a selected input.
    pub is_reference: bool,
    /// Complete observed selection cardinality across the rule's scope.
    pub scope_entries: usize,
    /// Current whole-scope result, independent of hypothetical path planning.
    pub scope_satisfied: bool,
}

pub(super) fn for_path(model: &RepositoryModel, path: &str) -> Vec<RepositoryFileExplanation> {
    model.repository_files.policies.iter().filter_map(|observed| {
        let rule = &observed.policy;
        let selected = rule.include.iter().any(|pattern| glob_matches(pattern, path))
            && !rule.exclude.iter().any(|pattern| glob_matches(pattern, path));
        let is_reference = matches!(&rule.predicate, RepositoryFilePredicate::BytesEqual { other } if other == path);
        (selected || is_reference).then(|| RepositoryFileExplanation {
            policy_id: observed.policy_id.clone(),
            policy: rule.clone(),
            claim: observed.claim.clone(),
            checkout_path: observed.checkout_path.clone(),
            entry: observed.entries.iter().find(|entry| entry.path == path).cloned()
                .or_else(|| observed.reference.as_ref().filter(|entry| entry.path == path).cloned()),
            is_reference,
            scope_entries: observed.entries.len(),
            scope_satisfied: observed.satisfied,
        })
    }).collect()
}

pub(super) fn display(policies: &[RepositoryFileExplanation]) -> String {
    if policies.is_empty() {
        return "<none>".into();
    }
    policies.iter().map(|policy| format!(
        "{}: {:?}; include {:?}; exclude {:?}; {:?}; {} selected; satisfied {}; checkout {:?}; reason {}",
        policy.policy_id, policy.policy.predicate, policy.policy.include, policy.policy.exclude,
        policy.policy.entry, policy.scope_entries, policy.scope_satisfied, policy.checkout_path,
        policy.policy.reason,
    )).collect::<Vec<_>>().join("; ")
}
