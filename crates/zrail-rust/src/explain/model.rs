//! Serializable effective policy for one repository-relative path.

use serde::{Deserialize, Serialize};

/// Effective architecture policy for one repository-relative path.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PathExplanation {
    /// The schema version of this serialized explanation.
    pub schema: u64,
    /// The normalized repository-relative path.
    pub path: String,
    /// The source class: `facade`, `implementation`, `test`, `auxiliary`,
    /// `entrypoint`, or `generated`.
    pub file_class: String,
    /// The source-graph reachability: `unreachable`, `test-only`, `production`, or `both`.
    pub reachability: String,
    /// The most specific Cargo package containing the path, when one exists.
    pub package: Option<String>,
    /// The dependency layer assigned to the package, when one matches.
    pub layer: Option<String>,
    /// The effect-profile names applied by the matched layer.
    pub profiles: Vec<String>,
    /// File and fact reachability used by each applied effect profile.
    #[serde(default)]
    pub profile_reachability: Vec<String>,
    /// The names of source scopes that include the path.
    pub scopes: Vec<String>,
    /// The matched layer and every layer it may depend on.
    pub permitted_dependency_layers: Vec<String>,
    /// The layer's external-dependency mode: `allow`, `locked`, or `none`.
    pub external_dependencies: Option<String>,
    /// Effect names denied by the matched layer's profiles.
    pub denied_effects: Vec<String>,
    /// Symbol paths denied by matching source scopes.
    pub denied_symbols: Vec<String>,
    /// Method names denied throughout Rust source.
    pub denied_methods: Vec<String>,
    /// Macro names denied throughout Rust source.
    pub denied_macros: Vec<String>,
    /// The macro-expansion mode: `allow` or `deny-unreviewed`.
    pub macro_expansion: String,
    /// Macro policy names whose expansions the contract permits.
    pub allowed_macro_expansions: Vec<String>,
    /// Allowed macro policy names whose inputs remain opaque.
    pub opaque_macro_inputs: Vec<String>,
    /// Observed repository macro implementations bound into the lock.
    pub content_bound_macro_implementations: Vec<String>,
    /// Observed macro spellings, preferred policy names, and independent origins.
    #[serde(default)]
    pub macro_invocations: Vec<MacroInvocationExplanation>,
    /// The unsafe-code mode: `allow` or `deny`.
    pub unsafe_code: String,
    /// The lint-suppression mode: `allow`, `reasoned`, or `deny`.
    pub lint_suppressions: String,
    /// The required sibling test path, when sibling tests apply to the path.
    pub expected_sibling_test: Option<String>,
    /// Invariant identifiers whose documents or evidence mention the path.
    pub invariants: Vec<String>,
    /// Capability-owner rules whose declared boundary contains the path.
    pub capability_owners: Vec<CapabilityOwnerExplanation>,
    /// Call-owner rules whose declared boundary contains the path.
    pub call_owners: Vec<CallOwnerExplanation>,
    /// The advisory line target for the source class, when configured.
    pub design_target: Option<usize>,
    /// The enforced line ceiling for the source class, when configured.
    pub hard_ceiling: Option<usize>,
    /// Whether a facade or entry point must remain declarative, when applicable.
    pub declarative_shape: Option<bool>,
    /// Whether the path must contain module-level documentation.
    pub module_docs_required: bool,
    /// Whether production tests must use sibling test modules.
    pub sibling_tests_required: bool,
}

/// One observed macro invocation identity as shown by `zrail explain`.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MacroInvocationExplanation {
    /// The exact path spelling at the invocation site.
    pub written: String,
    /// The stable user-spellable policy name, when resolution found one.
    pub preferred: Option<String>,
    /// Compiler, repository, dependency, or unresolved origins, separate from the name.
    pub origins: Vec<String>,
}

/// A capability-owner rule as it applies to an explained path.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityOwnerExplanation {
    /// The owner rule's unique name.
    pub name: String,
    /// The canonical capability path controlled by the owner.
    pub capability: String,
    /// The repository-relative paths allowed to use the capability.
    pub allow: Vec<String>,
    /// Whether the explained path is one of the allowed paths.
    pub allowed_here: bool,
    /// The source reachability evaluated by this owner.
    #[serde(default)]
    pub reachability: String,
    /// The human-authored reason for the ownership boundary.
    pub reason: String,
}

/// A call-owner rule as it applies to an explained path.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CallOwnerExplanation {
    /// The owner rule's unique name.
    pub name: String,
    /// The canonical function or method call controlled by the owner.
    pub call: String,
    /// The repository-relative paths allowed to make the call.
    pub allow: Vec<String>,
    /// Whether the explained path is one of the allowed paths.
    pub allowed_here: bool,
    /// The source reachability evaluated by this owner.
    #[serde(default)]
    pub reachability: String,
    /// The human-authored reason for the ownership boundary.
    pub reason: String,
}
