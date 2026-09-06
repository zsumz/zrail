//! Frozen dependency fields govern authored declarations independently of resolved package topology.

#[path = "legacy.rs"]
mod legacy;
#[path = "mutations.rs"]
mod mutations;

use super::metadata::Suite;

fn suite() -> Suite {
    Suite {
        name: "dependency-fields",
        policy_count: 30,
        input_count: 6,
        fixture_count: 151,
        positive_count: 34,
        extra_inputs: &["guardrails.toml"],
        legacy: legacy::check,
        mutations: mutations::cases,
        limitations: &[
            "Authored declaration versions, inheritance, optional/default feature flags, ordered feature arrays, publication lists, typed field projections, and required parser inputs only.",
            "Complete original assertion bodies and parser/read helpers execute unchanged. workspace_root is supplied from the isolated input path and unused registry fields are outside the minimal harness.",
            "All five manifest inputs and the original registry are bound. Publication equality proves the original conjunction of array type, length one, and first-item registry identity.",
            "No Cargo execution or resolution, whole-lock provenance, source behavior, or full repository qualification is claimed.",
        ],
    }
}

#[cfg(test)]
#[path = "qualification_test.rs"]
mod qualification_test;
