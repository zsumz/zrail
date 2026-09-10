//! Raw spelling predicates preserve literal scope independently of parsed configuration and execution.

#[path = "legacy.rs"]
mod legacy;
#[path = "mutations.rs"]
mod mutations;

use super::metadata::Suite;

fn suite() -> Suite {
    Suite {
        name: "raw-dependency",
        policy_count: 16,
        input_count: 4,
        fixture_count: 157,
        positive_count: 85,
        extra_inputs: &["guardrails.toml"],
        legacy: legacy::check,
        mutations: mutations::cases,
        limitations: &[
            "Raw UTF-8 lock-name lines, CI marker multiplicity/presence, exact trimmed attributes lines, and required input reads only.",
            "Original complete test bodies, raw name extraction, and read/registry helpers execute unchanged; workspace_root alone is supplied from the isolated input path.",
            "Lock bans inspect all text, including unrelated sections and multiline strings. Trailing comments and different assignment spacing retain the original extractor's behavior.",
            "CI markers inside comments or irrelevant fields deliberately retain original acceptance. These policies make no YAML structure or execution claim; separate workflow and execution requirements remain open.",
            "Absence bans do not themselves require files. Explicit UTF-8 self-equality policies retain missing-input and read preconditions. No Cargo graph or source certificate is produced.",
        ],
    }
}

#[cfg(test)]
#[path = "qualification_test.rs"]
mod qualification_test;
