//! Frozen authored dependency key sets retain their exact versus subset distinction.

#[path = "legacy.rs"]
mod legacy;
#[path = "mutations.rs"]
mod mutations;

use super::metadata::Suite;

fn suite() -> Suite {
    Suite {
        name: "key-sets",
        policy_count: 5,
        input_count: 6,
        fixture_count: 130,
        positive_count: 22,
        extra_inputs: &["guardrails.toml"],
        legacy: legacy::check,
        mutations: mutations::cases,
        limitations: &[
            "Authored ordinary-dependency key inventories only; no Cargo resolution, source reachability, or full repository claim.",
            "Original assertion bodies and manifest helpers execute unchanged. workspace_root is supplied from the isolated path; unused registry fields are outside this minimal harness.",
            "All five frozen manifests and the immutable original guardrail registry are bound. Only the selected manifest is mutated in each fixture.",
            "The simulator's original function also checks its workspace criticality version; that unmodified input is retained without claiming separate version-assertion qualification.",
        ],
    }
}

#[cfg(test)]
#[path = "qualification_test.rs"]
mod qualification_test;
