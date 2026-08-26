//! Public schema for governed-surface coverage reports.

mod common;
mod dependencies;
mod owners;
mod report;
mod source_policies;
mod test_mirrors;

pub use common::{GovernedCompilationDomain, GovernedFeaturePackage, GovernedFeatureWorld};
pub use dependencies::{GovernedDependencyPath, GovernedDependencyRule, GovernedPackageIdentity};
pub use owners::{GovernedOperationOccurrence, GovernedOwnerRule};
pub use report::{GovernedAnalysis, GovernedSurfaceReport};
pub use source_policies::{GovernedSourcePolicyOccurrence, GovernedSourcePolicyRail};
pub use test_mirrors::GovernedTestMirror;
