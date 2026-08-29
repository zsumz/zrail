//! Ratchet values compare current debt with declared and locked authority.

use zrail_core::{Finding, FindingSink, LockedRatchet, RatchetContract};

use crate::source::RustFileFacts;

use super::CountRatchetSpec;

pub(super) fn check_value(
    file: &RustFileFacts,
    value: usize,
    ratchet: Option<&RatchetContract>,
    locked: Option<&LockedRatchet>,
    spec: CountRatchetSpec<'_>,
    findings: &mut FindingSink,
    report_unratcheted: &impl Fn(&RustFileFacts, &mut FindingSink),
) {
    if value == 0 {
        if ratchet.is_some() && spec.report_source_lock_drift {
            findings.push(
                ratchet_finding(
                    spec,
                    &file.relative,
                    format!(
                        "{} was removed but the source retains a stale ratchet",
                        spec.debt
                    ),
                )
                .with_help("remove the ratchet from zrail.toml and run `zrail update`"),
            );
        }
        return;
    }
    let Some(ratchet) = ratchet else {
        report_unratcheted(file, findings);
        return;
    };
    let Some(locked) = locked else {
        if spec.report_source_lock_drift {
            findings.push(
                ratchet_finding(
                    spec,
                    &file.relative,
                    format!("reviewed {} ratchet is absent from zrail.lock", spec.debt),
                )
                .because(&ratchet.reason)
                .with_help("run `zrail update` and review the generated debt"),
            );
        }
        return;
    };
    if value > locked.value {
        findings.push(
            ratchet_finding(
                spec,
                &file.relative,
                format!(
                    "{} grew from the {}-construct ratchet to {value}",
                    spec.debt, locked.value
                ),
            )
            .because(&ratchet.reason),
        );
    } else if value < locked.value && spec.report_source_lock_drift {
        findings.push(
            ratchet_finding(
                spec,
                &file.relative,
                format!(
                    "{} shrank to {value} constructs but the lock still permits {}",
                    spec.debt, locked.value
                ),
            )
            .with_help("run `zrail update` to tighten the recorded debt"),
        );
    }
}

pub(super) fn ratchet_finding(spec: CountRatchetSpec<'_>, path: &str, message: String) -> Finding {
    Finding::error(spec.finding_id, spec.finding_rule, spec.category, message).at(path, None)
}
