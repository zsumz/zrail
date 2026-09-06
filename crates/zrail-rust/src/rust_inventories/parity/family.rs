//! Frozen syntax assertions share input handling while retaining distinct quantity/set oracles.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use super::{expression_mutations, legacy, mutations, owner_mutations};

#[derive(Clone, Copy)]
pub(super) enum Family {
    Methods,
    ExpressionPaths,
    Owners,
}

impl Family {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Methods => "methods",
            Self::ExpressionPaths => "expression-paths",
            Self::Owners => "owners",
        }
    }

    pub(super) fn report_env(self) -> &'static str {
        match self {
            Self::Methods => "ZRAIL_RC9_TRANSPORT_METHODS_REPORT",
            Self::ExpressionPaths => "ZRAIL_RC9_TRANSPORT_EXPRESSION_PATHS_REPORT",
            Self::Owners => "ZRAIL_RC9_TRANSPORT_OWNERS_REPORT",
        }
    }

    pub(super) fn totals(self) -> (usize, usize) {
        match self {
            Self::Methods => (76, 7),
            Self::ExpressionPaths => (83, 27),
            Self::Owners => (46, 15),
        }
    }

    pub(super) fn expected(self) -> BTreeMap<String, usize> {
        match self {
            Self::Methods => legacy::expected(),
            Self::ExpressionPaths => legacy::expected_associated(),
            Self::Owners => members(legacy::api::expected_owner_files()),
        }
    }

    pub(super) fn observed(self, path: &str, source: &str) -> BTreeMap<String, usize> {
        match self {
            Self::Methods => legacy::observed(path, source),
            Self::ExpressionPaths => legacy::associated(path, source),
            Self::Owners => members(legacy::owners(path, source)),
        }
    }

    pub(super) fn repository(self, root: &Path, roots: &[String]) -> BTreeMap<String, usize> {
        match self {
            Self::Methods => legacy::repository(root, roots),
            Self::ExpressionPaths => legacy::repository_associated(root, roots),
            Self::Owners => members(legacy::api::repository_owner_files(root, roots)),
        }
    }

    pub(super) fn check(self, counts: BTreeMap<String, usize>) {
        match self {
            Self::Methods => legacy::check_selector_methods(counts),
            Self::ExpressionPaths => legacy::check_associated(counts),
            Self::Owners => legacy::check_owners(member_files(counts)),
        }
    }

    pub(super) fn check_detector(self, counts: BTreeMap<String, usize>) {
        match self {
            Self::Methods => legacy::check_detector_methods(counts),
            Self::ExpressionPaths => legacy::check_detector_associated(counts),
            Self::Owners => legacy::check_detector_owners(member_files(counts)),
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
            Self::Owners => owner_mutations::cases(sources, roots),
        }
    }

    pub(super) fn legacy_measure(self) -> Option<super::model::LegacyMeasure> {
        matches!(self, Self::Owners).then_some(super::model::LegacyMeasure::DistinctOwnerFiles)
    }

    pub(super) fn native_counts(
        self,
        report: &crate::GovernedRustInventory,
    ) -> BTreeMap<String, usize> {
        report
            .counts
            .iter()
            .map(|count| {
                let quantity = if matches!(self, Self::Owners) {
                    assert!(count.count > 0, "observed ownership requires presence");
                    1
                } else {
                    count.count
                };
                (format!("{}:{}", count.path, count.name), quantity)
            })
            .collect()
    }
}

fn members(files: BTreeSet<String>) -> BTreeMap<String, usize> {
    files
        .into_iter()
        .map(|path| (format!("{path}:ConnectionSet"), 1))
        .collect()
}

fn member_files(counts: BTreeMap<String, usize>) -> BTreeSet<String> {
    counts
        .into_iter()
        .map(|(key, count)| {
            assert_eq!(
                count, 1,
                "distinct owner membership, not source occurrence quantity"
            );
            key.strip_suffix(":ConnectionSet")
                .expect("original written subject")
                .into()
        })
        .collect()
}
