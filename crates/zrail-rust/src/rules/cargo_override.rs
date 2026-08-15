//! Unsupported Cargo resolution surfaces fail closed before policy evaluation.

use zrail_core::{AnalysisQuality, Finding, FindingSink};

use super::RuleContext;

pub(super) fn evaluate(context: &RuleContext<'_>, findings: &mut FindingSink) {
    for resolution in &context.cargo.resolution_overrides {
        findings.push(
            Finding::error(
                "CARGO-OVERRIDE-001",
                "cargo.resolution-override",
                "dependency",
                format!(
                    "repository uses a Cargo resolution surface zrail does not attest: {}",
                    resolution.surface
                ),
            )
            .at(&resolution.path, None)
            .with_analysis(AnalysisQuality::Unresolved)
            .with_help(
                "remove the override or wait for zrail to attest its effective Cargo resolution",
            ),
        );
    }
}
