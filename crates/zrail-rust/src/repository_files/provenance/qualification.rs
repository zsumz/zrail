//! Frozen provenance metadata stays independent of whole-lock identity and execution claims.

#[path = "legacy.rs"]
mod legacy;
#[path = "mutations.rs"]
mod mutations;

use super::metadata::Suite;

fn suite() -> Suite {
    Suite {
        name: "provenance",
        policy_count: 27,
        input_count: 4,
        fixture_count: 161,
        positive_count: 39,
        extra_inputs: &["guardrails.toml"],
        legacy: legacy::check,
        mutations: mutations::cases,
        limitations: &[
            "Authored protocol version strings, exact workspace-reference key sets, typed inheritance/optional values, entire root override-key absence, and required TOML parser inputs only.",
            "The complete original no-override assertion body and helpers execute unchanged. Parser-input policies execute the original read/parse helper directly.",
            "Key inventories preserve the conjunction of exact table cardinality and required workspace/optional fields; typed value policies remain separate requirements.",
            "A zero-match override prohibition does not itself require a file; the explicit parser-input policies retain required input presence in the complete fragment.",
            "The source registry is immutable generator authority. Registry version-prefix assertions, Cargo.lock package traversal/provenance, execution, and full repository analysis are separate qualifications.",
        ],
    }
}

#[cfg(test)]
#[path = "qualification_test.rs"]
mod qualification_test;
