//! Completeness-certificate authority is exact; diagnostic counts are advisory.

use zrail_core::{Finding, FindingSink, LockFile};

pub(super) fn compare(current: &LockFile, candidate: &LockFile, findings: &mut FindingSink) {
    let Some(expected) = &candidate.analysis else {
        return;
    };
    let Some(reviewed) = &current.analysis else {
        findings.push(
            Finding::error(
                "LOCK-027",
                "lock.analysis",
                "lock",
                "zrail.lock has no reviewed analysis completeness certificate",
            )
            .with_help("run `zrail migrate-lock` before updating epoch-3 lock authority"),
        );
        return;
    };
    changed(
        reviewed.inventory_sha256 != expected.inventory_sha256,
        "LOCK-028",
        "the active analyzed inventory differs from zrail.lock",
        findings,
    );
    changed(
        reviewed.exclusions_sha256 != expected.exclusions_sha256,
        "LOCK-029",
        "the analyzed exclusion set differs from zrail.lock",
        findings,
    );
    changed(
        reviewed.cargo_lock_sha256 != expected.cargo_lock_sha256,
        "LOCK-039",
        "the exact Cargo.lock bytes differ from zrail.lock",
        findings,
    );
    changed(
        reviewed.analyzer_semantics != expected.analyzer_semantics,
        "LOCK-030",
        "the analysis certificate uses different analyzer semantics",
        findings,
    );
    changed(
        reviewed.contract_sources != expected.contract_sources,
        "LOCK-031",
        "the exact contract-source census differs from zrail.lock",
        findings,
    );
}

fn changed(changed: bool, id: &str, message: &str, findings: &mut FindingSink) {
    if changed {
        findings.push(
            Finding::error(id, "lock.analysis", "lock", message)
                .with_help("run `zrail diff` and review the changed analysis universe"),
        );
    }
}
