//! Exact lock drift comparison for dependencies, provenance, engine state, and ratchets.

mod analysis;
mod gates;
mod generated;
mod item_macros;
mod macros;
mod packages;
mod receipts;

use std::collections::BTreeMap;

use zrail_core::{
    Contract, DependencyMode, Finding, FindingSink, LOCK_SCHEMA, LOCK_SEMANTICS, LockFile,
};

use super::model::RepositoryModel;

pub(super) fn check_lock(
    model: &RepositoryModel,
    current: Option<&LockFile>,
    candidate: &LockFile,
    findings: &mut FindingSink,
) {
    if !requires_lock(&model.bundle.contract) {
        return;
    }
    let Some(current) = current else {
        findings.push(
            Finding::error(
                "LOCK-001",
                "lock.required",
                "lock",
                "declared exact or ratcheted architecture state requires zrail.lock",
            )
            .with_help("run `zrail update` and review the resolved architecture state"),
        );
        return;
    };
    if !current.has_supported_schema() {
        findings.push(
            Finding::error(
                "LOCK-020",
                "lock.schema",
                "lock",
                format!(
                    "zrail.lock uses schema {}, latest supported schema is {}",
                    current.schema, LOCK_SCHEMA
                ),
            )
            .with_help("use a zrail engine that understands this lock schema"),
        );
    }
    if !current.has_current_semantics() {
        findings.push(
            Finding::error(
                "LOCK-008",
                "lock.semantics",
                "lock",
                format!(
                    "zrail.lock uses semantics {}, current engine uses semantics {}",
                    current.semantics, LOCK_SEMANTICS
                ),
            )
            .with_help("review the semantic migration with a compatible zrail engine"),
        );
    }
    if current.contract_sha256 != model.bundle.sha256 {
        findings.push(
            Finding::error(
                "LOCK-002",
                "lock.contract",
                "lock",
                "zrail.lock was produced from different contract bytes",
            )
            .with_help("run `zrail diff` before updating the lock"),
        );
    }
    compare_locks(current, candidate, findings);
}

fn compare_locks(current: &LockFile, candidate: &LockFile, findings: &mut FindingSink) {
    analysis::compare(current, candidate, findings);
    packages::compare(current, candidate, findings);
    generated::compare(current, candidate, findings);
    gates::compare(current, candidate, findings);
    receipts::compare(current, candidate, findings);
    macros::compare(current, candidate, findings);
    item_macros::compare(current, candidate, findings);
    compare_ratchets(current, candidate, findings);
}

pub(super) fn requires_lock(contract: &Contract) -> bool {
    contract.dependencies.mode == DependencyMode::Locked
        || !contract.source.rust.generated.is_empty()
        || !contract.gates.is_empty()
        || !contract.source.rust.test_mirrors.is_empty()
        || !contract.source.rust.macros.allow.is_empty()
        || contract
            .source
            .rust
            .item_macros
            .iter()
            .any(|allowance| allowance.manifest.is_some())
        || !contract.ratchets.is_empty()
}

fn compare_ratchets(current: &LockFile, candidate: &LockFile, findings: &mut FindingSink) {
    let old = ratchet_values(current);
    let new = ratchet_values(candidate);
    for identity in new.keys().filter(|identity| !old.contains_key(*identity)) {
        let identity = ratchet_label(identity);
        findings.push(Finding::error(
            "LOCK-009",
            "lock.ratchet",
            "lock",
            format!("repository ratchet {identity:?} is not reviewed in zrail.lock"),
        ));
    }
    for identity in old.keys().filter(|identity| !new.contains_key(*identity)) {
        let identity = ratchet_label(identity);
        findings.push(Finding::error(
            "LOCK-007",
            "lock.ratchet",
            "lock",
            format!("zrail.lock retains stale ratchet {identity:?}"),
        ));
    }
    for (identity, old_value) in &old {
        let Some(new_value) = new.get(identity) else {
            continue;
        };
        if old_value != new_value {
            let identity = ratchet_label(identity);
            findings.push(Finding::error(
                "LOCK-010",
                "lock.ratchet",
                "lock",
                format!(
                    "ratchet {identity:?} records {old_value} but repository \
                     resolves to {new_value}"
                ),
            ));
        }
    }
}

type RatchetIdentity = (String, Option<String>, String);

fn ratchet_values(lock: &LockFile) -> BTreeMap<RatchetIdentity, usize> {
    lock.ratchets
        .iter()
        .map(|ratchet| {
            (
                (
                    ratchet.rule.clone(),
                    ratchet
                        .selector
                        .as_deref()
                        .map(zrail_core::normalize_ratchet_selector),
                    ratchet.target.clone(),
                ),
                ratchet.value,
            )
        })
        .collect()
}

fn ratchet_label((rule, selector, target): &RatchetIdentity) -> String {
    selector.as_ref().map_or_else(
        || format!("{rule}:{target}"),
        |selector| format!("{rule}[{selector}]:{target}"),
    )
}

#[cfg(test)]
#[path = "lock_compare_test.rs"]
mod lock_compare_test;
