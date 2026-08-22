//! Closed configuration modes with unambiguous semantics.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// A side-effect capability that a profile may forbid.
pub enum Effect {
    /// Runtime reads or writes through the filesystem.
    Filesystem,
    /// Filesystem access performed while compiling or generating code.
    CompileFilesystem,
    /// Runtime or build-time network access.
    Network,
    /// Starting or controlling operating-system processes.
    Process,
    /// Locking or other synchronization primitives.
    Synchronization,
    /// Creating or coordinating operating-system threads.
    Thread,
    /// Reading wall-clock time.
    WallClock,
    /// Depending on an asynchronous runtime.
    AsyncRuntime,
    /// Accessing a database client or server.
    Database,
    /// Starting or controlling container runtimes.
    ContainerRuntime,
    /// Reading or mutating runtime environment variables.
    Environment,
    /// Reading environment variables during compilation.
    CompileEnvironment,
    /// Reading nondeterministic random values.
    Randomness,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Policy for how declared workspace membership must match discovered membership.
pub enum ExactMode {
    /// Require the declared and discovered sets to match exactly.
    Exact,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// A binary permission policy.
pub enum PolicyMode {
    /// Permit the governed condition.
    Allow,
    /// Reject the governed condition.
    Deny,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Policy governing source-level lint suppression attributes.
pub enum LintSuppressionMode {
    /// Permit lint suppressions without a justification.
    Allow,
    /// Require each suppression to carry an accepted reason.
    Reasoned,
    /// Reject lint suppressions.
    Deny,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Policy governing unreviewed procedural-macro expansion.
pub enum MacroExpansionMode {
    #[default]
    /// Permit macro expansion without an allow-list entry.
    Allow,
    /// Require each procedural macro to match a reviewed allow-list entry.
    DenyUnreviewed,
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Inspection policy for the token input passed to an allowed macro.
pub enum MacroInputMode {
    #[default]
    /// Include invocation input when assessing the macro boundary.
    Inspect,
    /// Treat invocation input as opaque and trust only recorded provenance.
    Opaque,
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Confidence required when binding an observed macro invocation to an allowance.
pub enum MacroBindingMode {
    #[default]
    /// Require the macro candidate and its origin to resolve exactly.
    Exact,
    /// Permit the exact written spelling when static origin resolution is incomplete.
    Conservative,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Repository policy for symbolic links.
pub enum SymlinkMode {
    /// Reject every symbolic link.
    Deny,
    /// Permit links only when their resolved target remains inside the repository.
    Inside,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Source of authority for dependency topology.
pub enum DependencyMode {
    /// Require dependency observations to come from the lock state.
    Locked,
    /// Compare dependency topology observed directly from the workspace.
    Observed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Policy governing cycles in the package dependency graph.
pub enum CycleMode {
    /// Permit package dependency cycles.
    Allow,
    /// Reject package dependency cycles.
    Deny,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Policy governing module-level Rust documentation.
pub enum ModuleDocsMode {
    /// Permit modules without module-level documentation.
    Allow,
    /// Require module-level documentation on governed Rust modules.
    Required,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Structural policy for Rust facade files and entrypoints.
pub enum FacadeMode {
    /// Permit executable implementation logic in the governed facade.
    Allow,
    #[default]
    /// Require the governed facade to remain declarative.
    Declarative,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Placement policy for Rust tests.
pub enum TestMode {
    /// Permit tests in implementation files.
    Allow,
    /// Require tests in sibling test modules or integration-test directories.
    Sibling,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Policy for external dependencies used by a layer.
pub enum ExternalDependencyMode {
    /// Permit external dependencies without lock-state provenance.
    Allow,
    #[default]
    /// Permit external dependencies only when represented in lock state.
    Locked,
    /// Reject every external dependency.
    None,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Kind of source ownership boundary selected by an owner rule.
pub enum OwnerKind {
    /// Govern calls matching the owner selector.
    Call,
    /// Govern uses of a capability matching the owner selector.
    Capability,
    /// Govern files beneath directories matching the owner selector.
    Directory,
}
