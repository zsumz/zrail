//! Unexpanded Rust is an explicit, content-bound, reasoned trust boundary.

mod allowances;
mod binding_policy;
mod bindings;
mod diagnostics;
mod evaluation;
mod failure;
mod policy;
mod review;
mod source;

pub(crate) use binding_policy::build as binding_policy;
pub(super) use evaluation::evaluate;
pub(super) use policy::binds_allowance;
#[cfg(test)]
pub(crate) use policy::closes_source_operations;
pub(crate) use policy::{closes_async_syntax, closes_owned_operations, closes_type_duplication};

#[cfg(test)]
use policy::directly_inspected;
#[cfg(test)]
use review::{MacroBindingResult, candidate_names, review_without_definitions};

#[cfg(test)]
#[path = "macro_expansion_test.rs"]
mod macro_expansion_test;
