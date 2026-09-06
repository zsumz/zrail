//! Two frozen syntax assertions share trusted input handling while retaining distinct oracles.

use std::{collections::BTreeMap, path::Path};

use super::{expression_mutations, legacy, mutations};

#[derive(Clone, Copy)]
pub(super) enum Family {
    Methods,
    ExpressionPaths,
}

impl Family {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Methods => "methods",
            Self::ExpressionPaths => "expression-paths",
        }
    }

    pub(super) fn report_env(self) -> &'static str {
        match self {
            Self::Methods => "ZRAIL_RC9_TRANSPORT_METHODS_REPORT",
            Self::ExpressionPaths => "ZRAIL_RC9_TRANSPORT_EXPRESSION_PATHS_REPORT",
        }
    }

    pub(super) fn totals(self) -> (usize, usize) {
        match self {
            Self::Methods => (76, 7),
            Self::ExpressionPaths => (83, 27),
        }
    }

    pub(super) fn expected(self) -> BTreeMap<String, usize> {
        match self {
            Self::Methods => legacy::expected(),
            Self::ExpressionPaths => legacy::expected_associated(),
        }
    }

    pub(super) fn observed(self, path: &str, source: &str) -> BTreeMap<String, usize> {
        match self {
            Self::Methods => legacy::observed(path, source),
            Self::ExpressionPaths => legacy::associated(path, source),
        }
    }

    pub(super) fn repository(self, root: &Path, roots: &[String]) -> BTreeMap<String, usize> {
        match self {
            Self::Methods => legacy::repository(root, roots),
            Self::ExpressionPaths => legacy::repository_associated(root, roots),
        }
    }

    pub(super) fn check(self, counts: BTreeMap<String, usize>) {
        match self {
            Self::Methods => legacy::check_selector_methods(counts),
            Self::ExpressionPaths => legacy::check_associated(counts),
        }
    }

    pub(super) fn check_detector(self, counts: BTreeMap<String, usize>) {
        match self {
            Self::Methods => legacy::check_detector_methods(counts),
            Self::ExpressionPaths => legacy::check_detector_associated(counts),
        }
    }

    pub(super) fn cases(
        self,
        sources: &BTreeMap<String, String>,
        roots: &[String],
    ) -> Vec<mutations::Mutation> {
        match self {
            Self::Methods => mutations::cases(sources, roots),
            Self::ExpressionPaths => expression_mutations::cases(sources, roots),
        }
    }
}
