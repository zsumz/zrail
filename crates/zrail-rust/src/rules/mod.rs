//! Typed architecture rails evaluated over one repository fact model.

mod capability;
mod cargo_identity;
mod cargo_override;
pub(crate) mod count_ratchet;
mod dependency;
mod dependency_cycle;
mod dependency_deny;
mod dependency_paths;
mod evaluate;
pub(crate) mod evidence;
mod file_role;
pub(crate) mod generated;
mod hygiene;
mod macro_expansion;
mod repository;
mod size;
pub(crate) mod source_graph;
mod source_shape;
mod test_placement;
pub(crate) mod type_policy;

pub(crate) use capability::{
    CallOwnerEvidenceKind, assigned_profiles, async_syntax_name, matching_call_owner_evidence,
    matching_capability_owner, matching_operation_owner_operations,
};
pub(crate) use dependency_paths::{dependency_kind, resolve_denied_paths};
pub(crate) use evaluate::{RuleContext, evaluate};
pub(crate) use hygiene::glob_import_is_allowed;
pub(crate) use macro_expansion::binding_policy as binding_macro_policy;
pub(crate) use macro_expansion::closes_async_syntax;
pub(crate) use macro_expansion::closes_owned_operations;
#[cfg(test)]
pub(crate) use macro_expansion::closes_source_operations;
pub(crate) use macro_expansion::closes_type_duplication;
pub(crate) use repository::matching_directory_owner;
