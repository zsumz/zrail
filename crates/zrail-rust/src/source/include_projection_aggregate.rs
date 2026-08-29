//! Aggregate state stays separate from candidate traversal mechanics.

use std::collections::BTreeMap;

use zrail_core::AnalysisQuality;

use super::super::{
    AssociatedCandidateKind, GenericAssociatedCandidate, GenericRootShadow, ProjectionIdentity,
};

pub(in crate::source) struct CandidateAggregate {
    pub(in crate::source) instances: usize,
    pub(in crate::source) test_instances: usize,
    pub(in crate::source) quality: AnalysisQuality,
    pub(in crate::source) production: bool,
    pub(in crate::source) requires_projection: bool,
    pub(in crate::source) blocks_completeness: bool,
    pub(in crate::source) generic_shadow: Option<GenericRootShadow>,
    pub(in crate::source) associated_candidates:
        BTreeMap<(String, ProjectionIdentity, AssociatedCandidateKind), GenericAssociatedCandidate>,
}

impl Default for CandidateAggregate {
    fn default() -> Self {
        Self {
            instances: 0,
            test_instances: 0,
            quality: AnalysisQuality::Exact,
            production: false,
            requires_projection: false,
            blocks_completeness: false,
            generic_shadow: None,
            associated_candidates: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::source) struct TestCoverage {
    pub(in crate::source) instances: usize,
    pub(in crate::source) compatible: bool,
}

impl Default for TestCoverage {
    fn default() -> Self {
        Self {
            instances: 0,
            compatible: true,
        }
    }
}
