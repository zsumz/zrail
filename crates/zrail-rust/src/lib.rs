//! Rust and Cargo analysis for zrail repository architecture checks.
//!
//! This crate reads Cargo manifests, Rust source, [`zrail.toml`](https://github.com/zsumz/zrail)
//! contracts, and optional zrail locks as data. It does not invoke Cargo, build
//! scripts, procedural macros, qualification gates, or repository programs, and
//! its public operations do not write to the analyzed repository.
//!
//! [`check_repository`] is the primary integration point. It returns both a
//! diagnostic report and the independently observed candidate lock. [`build_lock`]
//! exposes that candidate directly for callers implementing an explicitly
//! authorized lock update. Relative configuration, lock, and explained paths are
//! interpreted beneath the supplied repository root.
//!
//! The baseline discovery types are public initialization support for the `zrail`
//! CLI. They describe conservative source roots and exact debt ratchets; they do
//! not modify a contract or lock themselves.

#![deny(missing_docs)]

mod cargo;
mod engine;
mod explain;
mod inventory;
mod onboarding;
mod rules;
mod source;
mod source_policy;

pub use engine::{
    CheckError, CheckResult, DoctorReport, build_lock, check_repository,
    check_repository_with_candidate_contract, check_repository_with_lock, doctor_repository,
};
pub use explain::{
    CallOwnerExplanation, CapabilityOwnerExplanation, ItemMacroAuthorityExplanation,
    MacroInvocationExplanation, PathExplanation, explain_path,
};
pub use onboarding::{
    BaselinePlan, BaselineRatchet, BaselineRule, BaselineSize, discover_baseline,
    discover_baseline_rules, discover_source_roots,
};
