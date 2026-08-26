//! Cfg analysis keeps predicate, guard, and completeness mechanics together.

use super::{SyntaxGuard, attributes, fact};

#[path = "cfg_completeness.rs"]
pub(super) mod cfg_completeness;
#[path = "cfg_guards.rs"]
pub(super) mod cfg_guards;
#[path = "cfg_logic.rs"]
mod cfg_logic;
#[path = "cfg_predicate.rs"]
mod cfg_predicate;
#[path = "cfg_predicate_text.rs"]
mod cfg_predicate_text;

pub(crate) use cfg_predicate::{CfgContext, CfgPredicate, CfgTruth};
