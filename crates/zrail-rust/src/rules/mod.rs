//! Typed architecture rails evaluated over one repository fact model.

mod capability;
mod cargo_override;
mod dependency;
mod dependency_cycle;
mod dependency_deny;
mod evaluate;
mod evidence;
pub(crate) mod generated;
mod hygiene;
mod repository;
mod size;
pub(crate) mod source_graph;
mod source_shape;
mod test_placement;

pub(crate) use evaluate::{RuleContext, evaluate};
