//! Existing measured ratchets additionally bind optional exact authored baselines.

use zrail_core::{Finding, FindingSink, LockedRatchet};

pub(super) fn check_ratchet(
    file: &crate::source::RustFileFacts,
    contract_ratchet: &zrail_core::RatchetContract,
    locked_ratchet: Option<&LockedRatchet>,
    findings: &mut FindingSink,
) {
    if let Some(baseline) = contract_ratchet.baseline
        && file.lines != baseline
    {
        findings.push(
            Finding::error(
                "RUST-SIZE-009",
                "rust.file-size.baseline",
                "source-size",
                format!(
                    "source is {} lines but its exact authored baseline is {baseline}",
                    file.lines
                ),
            )
            .at(&file.relative, None)
            .because(&contract_ratchet.reason)
            .with_help(
                "split growing source; tighten a shrinking baseline in the contract and lock",
            ),
        );
    }
    let Some(locked_ratchet) = locked_ratchet else {
        findings.push(
            Finding::error(
                "RUST-SIZE-003",
                "rust.file-size.ratchet",
                "source-size",
                "reviewed ratchet is absent from zrail.lock",
            )
            .at(&file.relative, None)
            .because(&contract_ratchet.reason)
            .with_help("run `zrail update` and review the generated debt"),
        );
        return;
    };
    if file.lines > locked_ratchet.value {
        findings.push(
            Finding::error(
                "RUST-SIZE-003",
                "rust.file-size.ratchet",
                "source-size",
                format!(
                    "source grew from its {}-line ratchet to {} lines",
                    locked_ratchet.value, file.lines
                ),
            )
            .at(&file.relative, None)
            .because(&contract_ratchet.reason),
        );
    } else if file.lines < locked_ratchet.value {
        findings.push(
            Finding::error(
                "RUST-SIZE-004",
                "rust.file-size.ratchet",
                "source-size",
                format!(
                    "source shrank to {} lines but lock still permits {}",
                    file.lines, locked_ratchet.value
                ),
            )
            .at(&file.relative, None)
            .with_help("run `zrail update` to tighten the recorded ceiling"),
        );
    }
}
