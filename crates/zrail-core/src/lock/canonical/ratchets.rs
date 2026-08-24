//! Canonical validation and ordering for selector-aware ratchets.

use super::super::{LockError, LockFile};
use super::ensure_unique;

pub(super) fn canonicalize(lock: &mut LockFile) -> Result<(), LockError> {
    for ratchet in &mut lock.ratchets {
        if ratchet.rule.trim().is_empty() || ratchet.target.trim().is_empty() {
            return Err(LockError(
                "locked ratchets require non-empty rule and target".into(),
            ));
        }
        if ratchet.value == 0 {
            return Err(LockError(format!(
                "locked ratchet {}:{} must be positive",
                ratchet.rule, ratchet.target
            )));
        }
        if let Some(selector) = &mut ratchet.selector {
            if selector.trim().is_empty() {
                return Err(LockError(format!(
                    "locked ratchet {}:{} has an empty selector",
                    ratchet.rule, ratchet.target
                )));
            }
            *selector = crate::normalize_ratchet_selector(selector);
        }
    }
    lock.ratchets.sort_by(|left, right| {
        (&left.rule, &left.selector, &left.target).cmp(&(
            &right.rule,
            &right.selector,
            &right.target,
        ))
    });
    ensure_unique(
        lock.ratchets.iter().map(|ratchet| {
            format!(
                "{}:{}:{}",
                ratchet.rule,
                ratchet.selector.as_deref().unwrap_or("<none>"),
                ratchet.target
            )
        }),
        "locked ratchet",
    )
}
