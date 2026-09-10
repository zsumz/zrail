//! Language-neutral architecture contracts, lock state, diagnostics, and semantic diffs.
#![doc = include_str!("crate.md")]
#![deny(missing_docs)]
mod contract;
mod contract_edit;
mod diagnostic;
mod diff;
mod digest;
mod exports;
mod input;
mod lock;
mod migration;
mod path;
mod ratchet;
mod receipt;
mod report;

pub use exports::*;
#[cfg(test)]
#[path = "lock_analysis_test.rs"]
mod lock_analysis_test;
