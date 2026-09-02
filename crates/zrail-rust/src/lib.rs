//! Rust and Cargo analysis for zrail repository architecture checks.
//!
//! This crate reads Cargo manifests, Rust source, [`zrail.toml`](https://github.com/zsumz/zrail)
//! contracts, and optional zrail locks as data. It does not invoke Cargo, build
//! scripts, procedural macros, qualification gates, or repository programs, and
//! its public operations do not write to the analyzed repository.
//!
//! [`check_repository`] is the primary integration point. It returns both a
//! diagnostic report and, when analysis is complete, an independently observed
//! candidate lock. [`build_lock`] exposes that candidate directly for callers implementing an explicitly
//! authorized lock update. Relative configuration, lock, and explained paths are
//! interpreted beneath the supplied repository root.
//!
//! The baseline discovery types are public initialization support for the `zrail`
//! CLI. They describe conservative source roots and exact debt ratchets; they do
//! not modify a contract or lock themselves.

#![deny(missing_docs)]

mod analysis;
mod cargo;
mod coverage;
mod engine;
mod explain;
mod inventory;
mod mirror_execution;
mod mirror_inputs;
mod mirrors;
mod onboarding;
mod rules;
mod source;
mod source_policy;

#[cfg(test)]
#[path = "async_glob_policy_test.rs"]
mod async_glob_policy_test;

#[cfg(test)]
#[path = "type_policy_test.rs"]
mod type_policy_test;

#[cfg(test)]
#[path = "place_domain_world_test.rs"]
mod place_domain_world_test;

#[cfg(test)]
#[path = "type_policy_cfg_attr_test.rs"]
mod type_policy_cfg_attr_test;

#[cfg(test)]
#[path = "type_shape_test.rs"]
mod type_shape_test;

pub use analysis::{AnalysisIssue, AnalysisIssueKind, AnalysisMetrics, AnalysisOutcome};
pub use coverage::{
    GovernedAnalysis, GovernedCompilationDomain, GovernedDependencyPath, GovernedDependencyRule,
    GovernedFeaturePackage, GovernedFeatureWorld, GovernedOperationOccurrence, GovernedOwnerRule,
    GovernedPackageIdentity, GovernedSourcePolicyOccurrence, GovernedSourcePolicyRail,
    GovernedSurfaceReport, GovernedTestMirror, GovernedTypeField, GovernedTypeObservation,
    GovernedTypePolicy, governed_surface_report,
};
pub use engine::{
    CheckError, CheckResult, DoctorReport, build_lock, check_repository,
    check_repository_with_candidate_contract, check_repository_with_limit,
    check_repository_with_lock, doctor_repository,
};
pub use explain::{
    CallOwnerExplanation, CapabilityOwnerExplanation, ItemMacroAuthorityExplanation,
    MacroInvocationExplanation, PathExplanation, explain_hypothetical_path, explain_path,
};
pub use mirrors::{
    MirrorExecutionResult, MirrorPlan, MirrorReceiptBundle, MirrorResultSet, MirrorTestResult,
    MirrorVerification, PlannedTestMirror, RenderedMirrorReceipt, render_test_mirror_receipts,
    test_mirror_plan, verify_test_mirror_plan, verify_test_mirrors,
};
pub use onboarding::{
    BaselinePlan, BaselineRatchet, BaselineRule, BaselineSize, RepositorySelection,
    discover_baseline, discover_baseline_rules, discover_source_roots,
    discover_source_roots_with_selection,
};
