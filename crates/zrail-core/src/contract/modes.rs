//! Closed configuration modes with unambiguous semantics.

mod policy;

use serde::{Deserialize, Serialize};

pub use policy::{
    CycleMode, DependencyMode, ExactMode, ExternalDependencyMode, FacadeMode, GlobImportMode,
    LintSuppressionMode, MacroBindingMode, MacroExpansionBindings, MacroExpansionMode,
    MacroInputMode, ModuleDocsMode, OwnerKind, PolicyMode, SymlinkMode, TestMode,
};

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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Runtime-neutral asynchronous Rust syntax governed by a profile.
pub enum AsyncSyntax {
    /// An `async fn` declaration in a free, trait, or implementation context.
    AsyncFn,
    /// An `async { ... }` block.
    AsyncBlock,
    /// An `async || ...` closure.
    AsyncClosure,
    /// An `.await` suspension point.
    Await,
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Reviewed claim about async syntax emitted by an opaque macro expansion.
pub enum MacroAsyncSyntax {
    #[default]
    /// Expansion output remains opaque to async-syntax analysis.
    Opaque,
    #[serde(rename = "none")]
    /// Exact review attests that expansion output introduces no async syntax.
    None,
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Reviewed claim about Clone/Copy implementations emitted by a macro expansion.
pub enum MacroDuplicationEffect {
    #[default]
    /// Expansion output remains opaque to per-type duplication analysis.
    Opaque,
    #[serde(rename = "none")]
    /// Exact review attests that expansion output adds no Clone or Copy implementation.
    None,
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Reviewed claim about source operations emitted by a macro expansion.
pub enum MacroSourceOperations {
    #[default]
    /// Expansion output remains opaque to source-operation ownership analysis.
    Opaque,
    #[serde(rename = "none")]
    /// Exact review attests that expansion output introduces no source operations.
    None,
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Source reachability considered by an effect profile.
pub enum PolicyReachability {
    /// Evaluate effects in every Cargo target and syntax guard.
    #[default]
    All,
    /// Evaluate only ordinary facts reachable from a library or binary target.
    Production,
}
