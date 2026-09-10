//! Frozen workflow and script predicates retain raw matching and separate execution claims.

#[path = "legacy.rs"]
mod legacy;
#[path = "mutations.rs"]
mod mutations;

use super::metadata::Suite;

fn suite() -> Suite {
    Suite {
        name: "qualification",
        policy_count: 125,
        input_count: 24,
        fixture_count: 923,
        positive_count: 410,
        extra_inputs: &[],
        legacy: legacy::check,
        mutations: mutations::cases,
        limitations: &[
            "Raw whole-file literal presence, absence and counts, first-marker order/intervals, trimmed image-line allowlists, and required UTF-8 reads only.",
            "Complete original test bodies, image constants, and the read helper execute unchanged; only workspace_root and policy-to-test dispatch belong to the trusted harness.",
            "All raw markers include comments and string contents. These policies neither parse workflow structure nor prove that scripts, Cargo, smoke scenarios, or archive verification executed.",
            "The existing CI input-read policy is qualified separately in raw-dependency parity. KD-CI-ENFORCEMENT remains a blocker for the proposed enforced downstream zrail lane.",
            "Only the 24 exact qualification inputs are copied. Full downstream source analysis, combined policy qualification, behavioral runners and release execution evidence remain required.",
        ],
    }
}

#[cfg(test)]
#[path = "qualification_test.rs"]
mod qualification_test;
