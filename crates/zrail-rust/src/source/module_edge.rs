//! Exact module edges selected by reachable source-graph traversal.

use zrail_core::SourceSpan;

use super::{Reachability, SubmoduleBase};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ResolvedModuleEdge {
    pub(crate) parent: String,
    pub(crate) module_name: String,
    pub(crate) child: String,
    pub(crate) child_base: SubmoduleBase,
    pub(crate) reachability: Reachability,
    pub(crate) cfg_test: bool,
    pub(crate) span: Option<SourceSpan>,
}
