//! Typed architecture rails evaluated over one repository fact model.

mod capability;
mod cargo_identity;
mod cargo_override;
pub(crate) mod count_ratchet;
mod dependency;
mod dependency_cycle;
mod dependency_deny;
mod evaluate;
mod evidence;
mod file_role;
pub(crate) mod generated;
mod hygiene;
mod macro_expansion;
mod repository;
mod size;
pub(crate) mod source_graph;
mod source_shape;
mod test_placement;

pub(crate) use evaluate::{RuleContext, evaluate};
pub(crate) use macro_expansion::binding_policy as binding_macro_policy;
