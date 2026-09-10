//! Scoped line budgets and bounded, exact-file hard-ceiling authority.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Opt-in scoped budgets enforce hard ceilings independently of measured ratchets.
pub struct SizePolicyContract {
    /// Required accountability form for every above-hard exception in this policy.
    #[serde(default)]
    pub exception_metadata: SizeExceptionMetadata,
    /// Same-tier selectors; a physical file may match at most one override.
    #[serde(default)]
    pub overrides: Vec<ScopedBudgetContract>,
    /// Exact-file, explicitly bounded authority above an effective hard ceiling.
    #[serde(default)]
    pub exceptions: Vec<SizeExceptionContract>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Closed required-metadata choices preserve consumers' distinct exception contracts.
pub enum SizeExceptionMetadata {
    /// Require an owner/issue pair or a tracking label.
    #[default]
    OwnerIssueOrTracking,
    /// Require both an owner and an issue, even when a tracking label is supplied.
    OwnerIssue,
    /// Require a tracking label, even when an owner/issue pair is supplied.
    Tracking,
    /// Require all three accountability fields.
    OwnerIssueAndTracking,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One named package/path/role intersection overriding global role defaults.
pub struct ScopedBudgetContract {
    /// Stable policy name, independent of TOML order.
    pub name: String,
    /// Repository-relative globs selecting physical source paths.
    pub include: Vec<String>,
    /// Repository-relative globs subtracting paths from this selector.
    #[serde(default)]
    pub exclude: Vec<String>,
    /// Exact owning package names; empty selects every package.
    #[serde(default)]
    pub packages: Vec<String>,
    /// Effective size roles; empty selects every role.
    #[serde(default)]
    pub roles: Vec<SizeRole>,
    /// Complete replacement thresholds, without fieldwise inheritance.
    pub budget: SizeThresholds,
    /// Human justification for this source family's thresholds.
    pub reason: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Independent target, advisory soft threshold, and enforced hard ceiling.
pub struct SizeThresholds {
    /// Design target; excess is an error or warning according to `target_mode`.
    pub target: usize,
    /// Optional attention threshold; excess always produces a warning.
    #[serde(default)]
    pub soft: Option<usize>,
    /// Maximum without a separately reasoned, exact-file exception.
    pub hard: usize,
    /// Target severity; omission preserves the existing error behavior.
    #[serde(default)]
    pub target_mode: SizeTargetMode,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Whether an unratcheted design-target excess fails a check.
pub enum SizeTargetMode {
    /// Reject excess unless covered by an exact measured ratchet.
    #[default]
    Error,
    /// Report excess without failing a check or requiring a ratchet.
    Warn,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Budget selection role, independent of compilation and test execution identity.
pub enum SizeRole {
    /// Production facade selected by source structure policy.
    Facade,
    /// Explicit test facade or test-only `lib.rs`/`mod.rs`.
    TestFacade,
    /// Production implementation source.
    Implementation,
    /// Source reachable only in test compilation, excluding test facades.
    Test,
    /// Auxiliary source such as examples and build scripts.
    Auxiliary,
    /// Executable `main.rs` source.
    Entrypoint,
    /// Source covered by a reviewed generated-source contract.
    Generated,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Exact, bounded hard-ceiling exception; ordinary lock updates cannot create one.
pub struct SizeExceptionContract {
    /// Exact normalized repository-relative Rust source path.
    pub path: String,
    /// Explicit maximum line count, above the selected policy's hard ceiling.
    pub hard: usize,
    /// Justification for this above-hard debt, separate from any ratchet reason.
    pub reason: String,
    /// Accountable owner; required together with `issue` unless `tracking` is set.
    #[serde(default)]
    pub owner: Option<String>,
    /// Actionable issue identity; required together with `owner` unless tracking is set.
    #[serde(default)]
    pub issue: Option<String>,
    /// Reviewed tracking identity, used by policies without an owner/issue pair.
    #[serde(default)]
    pub tracking: Option<String>,
}
