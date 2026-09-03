//! Exact lock drift for package-bound macro implementation manifests.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{Finding, FindingSink, LockFile, LockedMacroImplementation, LockedMacroSource};

pub(super) fn compare(current: &LockFile, candidate: &LockFile, findings: &mut FindingSink) {
    compare_implementations(current, candidate, findings);
    compare_sources(current, candidate, findings);
}

fn compare_sources(current: &LockFile, candidate: &LockFile, findings: &mut FindingSink) {
    let old = sources(&current.macro_sources);
    let new = sources(&candidate.macro_sources);
    for allowance in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        compare_source_group(
            allowance,
            old.get(allowance).map(Vec::as_slice).unwrap_or_default(),
            new.get(allowance).map(Vec::as_slice).unwrap_or_default(),
            findings,
        );
    }
}

fn compare_source_group(
    allowance: &str,
    old: &[&LockedMacroSource],
    new: &[&LockedMacroSource],
    findings: &mut FindingSink,
) {
    if let ([before], [after]) = (old, new) {
        if before != after {
            findings.push(Finding::error(
                "LOCK-038",
                "lock.macro-source",
                "lock",
                format!(
                    "macro allowance {allowance:?} resolved package changed from {} to {}",
                    label(before),
                    label(after)
                ),
            ));
        }
        return;
    }
    let old = source_identities(old);
    let new = source_identities(new);
    for identity in old.keys().chain(new.keys()).collect::<BTreeSet<_>>() {
        let subject = format!("{allowance} [{identity}]");
        match (old.get(identity), new.get(identity)) {
            (None, Some(_)) => findings.push(Finding::error(
                "LOCK-036",
                "lock.macro-source",
                "lock",
                format!("macro allowance {subject:?} lacks locked Cargo package authority"),
            )),
            (Some(_), None) => findings.push(Finding::error(
                "LOCK-037",
                "lock.macro-source",
                "lock",
                format!("zrail.lock retains stale macro source {subject:?}"),
            )),
            (Some(before), Some(after)) if before != after => findings.push(Finding::error(
                "LOCK-038",
                "lock.macro-source",
                "lock",
                format!(
                    "macro allowance {subject:?} resolved package changed from {} to {}",
                    label(before),
                    label(after)
                ),
            )),
            _ => {}
        }
    }
}

fn sources(values: &[LockedMacroSource]) -> BTreeMap<&str, Vec<&LockedMacroSource>> {
    let mut sources = BTreeMap::<_, Vec<_>>::new();
    for source in values {
        sources
            .entry(source.allowance.as_str())
            .or_default()
            .push(source);
    }
    sources
}

fn source_identities<'a>(
    values: &[&'a LockedMacroSource],
) -> BTreeMap<String, &'a LockedMacroSource> {
    values
        .iter()
        .map(|source| (label(source), *source))
        .collect()
}

fn label(source: &LockedMacroSource) -> String {
    format!("{} {} ({})", source.package, source.version, source.source)
}

fn compare_implementations(current: &LockFile, candidate: &LockFile, findings: &mut FindingSink) {
    let old = implementations(&current.macro_implementations);
    let new = implementations(&candidate.macro_implementations);
    for identity in old
        .keys()
        .chain(new.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
    {
        match (old.get(&identity), new.get(&identity)) {
            (None, Some(_)) => findings.push(Finding::error(
                "LOCK-021",
                "lock.macro-implementation",
                "lock",
                format!(
                    "repository macro implementation {identity:?} is not reviewed in zrail.lock"
                ),
            )),
            (Some(_), None) => findings.push(Finding::error(
                "LOCK-022",
                "lock.macro-implementation",
                "lock",
                format!("zrail.lock retains stale repository macro implementation {identity:?}"),
            )),
            (Some(left), Some(right)) if left.inputs_sha256 != right.inputs_sha256 => {
                findings.push(Finding::error(
                    "LOCK-023",
                    "lock.macro-implementation",
                    "lock",
                    format!("reviewed repository macro implementation {identity:?} changed"),
                ));
            }
            _ => {}
        }
    }
}

fn implementations(
    values: &[LockedMacroImplementation],
) -> BTreeMap<String, &LockedMacroImplementation> {
    values
        .iter()
        .map(|value| (format!("{}:{}", value.directory, value.package), value))
        .collect()
}
