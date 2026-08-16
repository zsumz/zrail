//! Unsupported Cargo authority surfaces fail closed before policy evaluation.

use zrail_core::{AnalysisQuality, Finding, FindingSink};

use crate::cargo::CargoAuthorityKind;

use super::RuleContext;

pub(super) fn evaluate(context: &RuleContext<'_>, findings: &mut FindingSink) {
    for surface in &context.cargo.authority_surfaces {
        let finding = match surface.kind {
            CargoAuthorityKind::Resolution => Finding::error(
                "CARGO-OVERRIDE-001",
                "cargo.resolution-override",
                "dependency",
                format!(
                    "repository uses a Cargo resolution surface zrail does not attest: {}",
                    surface.surface
                ),
            )
            .with_help(
                "remove the override or wait for zrail to attest its effective Cargo resolution",
            ),
            CargoAuthorityKind::RepositoryConfiguration => Finding::error(
                "CARGO-CONFIG-001",
                "cargo.execution-configuration",
                "qualification",
                "repository-local Cargo configuration can alter qualification execution and is not permitted",
            )
            .with_help(
                "remove the Cargo configuration; qualification must use pinned repository inputs",
            ),
        };
        findings.push(
            finding
                .at(&surface.path, None)
                .with_analysis(AnalysisQuality::Unresolved),
        );
    }
}
