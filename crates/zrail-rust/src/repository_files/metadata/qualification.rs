//! Trusted file qualification shares input binding and executes only frozen test harnesses.

#[path = "fixtures.rs"]
pub(super) mod fixtures;
#[path = "legacy.rs"]
mod legacy;
#[path = "model.rs"]
pub(super) mod model;
#[path = "mutations.rs"]
pub(super) mod mutations;

#[cfg(test)]
#[path = "qualification_test.rs"]
mod qualification_test;

pub(super) use qualification_test::qualify;

use std::path::Path;
use zrail_core::RepositoryFileRule;

pub(super) struct Suite {
    pub(super) name: &'static str,
    pub(super) policy_count: usize,
    pub(super) input_count: usize,
    pub(super) fixture_count: usize,
    pub(super) positive_count: usize,
    pub(super) extra_inputs: &'static [&'static str],
    pub(super) legacy: fn(&Path, &RepositoryFileRule),
    pub(super) mutations: fn(&RepositoryFileRule, &[u8]) -> Vec<mutations::Mutation>,
    pub(super) limitations: &'static [&'static str],
}

pub(super) fn metadata() -> Suite {
    Suite {
        name: "metadata",
        policy_count: 35,
        input_count: 11,
        fixture_count: 134,
        positive_count: 35,
        extra_inputs: &[],
        legacy: |root, _| legacy::check(root),
        mutations: mutations::cases,
        limitations: &[
            "Authored metadata fields, UTF-8 license equality, physical file presence, and deliberate raw markers only; no resolved Cargo or execution claim.",
            "The original assertion body and helpers execute unchanged, with workspace_root supplied from the isolated input path.",
            "Only the declared 11 metadata inputs are copied; other source and independently governed assertions require full repository qualification.",
            "Malformed documents and exhausted bounds fail analysis explicitly instead of returning partial observations.",
        ],
    }
}
