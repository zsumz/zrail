//! Cross-cutting architecture-policy declarations.

use super::super::modes::{
    AsyncSyntax, Effect, ExternalDependencyMode, OwnerKind, PolicyReachability,
};
use super::{DependencyEdgeKind, DependencyReachability};
use serde::{Deserialize, Serialize};

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Named capability profile applied to one or more architecture layers."] pub struct ProfileContract {
    #[serde(default)]
    #[doc = "Source reachability to which this profile applies."] pub reachability: PolicyReachability,
    #[doc = "Side effects prohibited for packages using this profile."] pub effects: EffectBoundary,
    #[serde(default)]
    #[doc = "Runtime-neutral Rust syntax prohibited for packages using this profile."] pub syntax: SyntaxBoundary,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Set of side-effect capabilities prohibited by a profile."] pub struct EffectBoundary {
    #[serde(default)]
    #[doc = "Effects rejected when observed in a package using the profile."] pub deny: Vec<Effect>,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Set of runtime-neutral Rust syntax prohibited by a profile."] pub struct SyntaxBoundary {
    #[serde(default)]
    #[doc = "Async syntax rejected when observed in a package using the profile."] pub deny: Vec<AsyncSyntax>,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Named package layer and its permitted dependency boundaries."] pub struct LayerContract {
    #[doc = "Stable layer name referenced by other layer declarations."] pub name: String,
    #[doc = "Cargo package-name patterns assigned to this layer."] pub packages: Vec<String>,
    #[serde(default)]
    #[doc = "Layer names that packages in this layer may depend on."] pub may_depend_on: Vec<String>,
    #[serde(default)]
    #[doc = "Effect-profile names enforced for packages in this layer."] pub profiles: Vec<String>,
    #[doc = "Human explanation of the layer's architectural role."] pub reason: String,
    #[serde(default)]
    #[doc = "Policy for dependencies crossing this layer's repository boundary."] pub dependencies: LayerDependencies,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Dependency-source policy attached to an architecture layer."] pub struct LayerDependencies {
    #[serde(default)]
    #[doc = "Authority required for dependencies outside the workspace."] pub external: ExternalDependencyMode,
}

#[rustfmt::skip]
impl Default for LayerDependencies { fn default() -> Self { Self { external: ExternalDependencyMode::Locked } } }

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Named prohibition on selected package dependency edges."] pub struct DependencyRule {
    #[doc = "Stable rule name used in findings and semantic diffs."] pub name: String,
    #[doc = "Package-name pattern selecting dependency origins."] pub from: String,
    #[serde(default)]
    #[doc = "Package-name patterns rejected as dependency destinations."] pub deny: Vec<String>,
    #[serde(default)]
    #[doc = "Whether the denial covers immediate edges or every resolved dependency path."] pub reachability: DependencyReachability,
    #[serde(default)]
    #[doc = "First-edge Cargo dependency kinds covered by the rule; empty means every kind."] pub kinds: Vec<DependencyEdgeKind>,
    #[doc = "Human explanation of the prohibited dependency direction."] pub reason: String,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Named repository source scope with optional denied symbols."] pub struct ScopeContract {
    #[doc = "Stable scope name used in findings and semantic diffs."] pub name: String,
    #[doc = "Repository-relative patterns included in this scope."] pub include: Vec<String>,
    #[serde(default)]
    #[doc = "Repository-relative patterns removed from the included set."] pub exclude: Vec<String>,
    #[doc = "Human explanation of the scope boundary."] pub reason: String,
    #[serde(default)]
    #[doc = "Symbols prohibited within the resulting source set."] pub symbols: SymbolBoundary,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Denied symbol identities applied within a source scope."] pub struct SymbolBoundary {
    #[serde(default)]
    #[doc = "Fully qualified symbols rejected when used in the scope."] pub deny: Vec<String>,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Allow-list boundary for one governed source relationship."] pub struct OwnerContract {
    #[doc = "Stable owner-rule name used in findings and semantic diffs."] pub name: String,
    #[doc = "Kind of source relationship selected by this rule."] pub kind: OwnerKind,
    #[serde(default)] #[doc = "Source reachability to which this ownership rule applies."] pub reachability: PolicyReachability,
    #[serde(default)] #[doc = "Repository-relative patterns limiting where the rule is evaluated."] pub within: Vec<String>,
    #[serde(rename = "match")]
    #[doc = "Canonical source relationship identity governed by this rule."] pub selector: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[doc = "Written receiver methods treated as mutation for a field-mutation owner."] pub mutating_methods: Vec<String>,
    #[doc = "Package or source identities permitted to own the selected boundary."] pub allow: Vec<String>,
    #[doc = "Human explanation of the ownership boundary."] pub reason: String,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Tightening-only measured limit whose accepted value is stored in lock state."] pub struct RatchetContract {
    #[doc = "Measurement rule identifier understood by the active adapter."] pub rule: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[doc = "Optional normalized denied-operation selector measured independently."] pub selector: Option<String>,
    #[doc = "Repository-relative or package target measured by the rule."] pub target: String,
    #[doc = "Human explanation of why this metric may only tighten."] pub reason: String,
}
