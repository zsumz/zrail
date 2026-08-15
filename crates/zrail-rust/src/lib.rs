//! Rust and Cargo adapter for deterministic repository architecture checks.

mod cargo;
mod engine;
mod explain;
mod inventory;
mod onboarding;
mod rules;
mod source;

pub use engine::{
    CheckError, CheckResult, DoctorReport, build_lock, check_repository,
    check_repository_with_lock, doctor_repository,
};
pub use explain::{
    CallOwnerExplanation, CapabilityOwnerExplanation, PathExplanation, explain_path,
};
pub use inventory::{FileClass, RepositoryInventory, RepositoryInventoryError};
pub use onboarding::{BaselinePlan, BaselineRatchet, discover_baseline, discover_source_roots};
