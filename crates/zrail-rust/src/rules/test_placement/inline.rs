//! Inline-test debt uses the shared exact per-file count ratchet.

use zrail_core::{Finding, FindingSink};

use crate::source::RustFileFacts;

use super::super::{
    RuleContext,
    count_ratchet::{self, CountRatchetSpec},
};

pub(super) fn check(context: &RuleContext<'_>, findings: &mut FindingSink) {
    count_ratchet::evaluate(
        context,
        CountRatchetSpec {
            rule: "rust.inline-tests",
            finding_id: "RUST-TEST-005",
            finding_rule: "rust.tests.inline.ratchet",
            category: "test-placement",
            debt: "inline tests",
        },
        None,
        findings,
        report_unratcheted,
    );
}

fn report_unratcheted(file: &RustFileFacts, findings: &mut FindingSink) {
    for test in &file.tests {
        findings.push(
            Finding::error(
                "RUST-TEST-001",
                "rust.tests.sibling",
                "test-placement",
                format!("production source contains test construct {}", test.name),
            )
            .at(&file.relative, test.span)
            .with_help("move the proof into a reachable sibling `_test.rs` module"),
        );
    }
}
