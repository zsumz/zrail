//! Public audit schema for exact Rust type policy observations.

use serde::Serialize;
use zrail_core::{AnalysisQuality, SourceSpan};

use super::GovernedCompilationDomain;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
/// One exact Rust type policy with authored expectations and observed source facts.
pub struct GovernedTypePolicy {
    /// Canonical report identity for this policy.
    pub policy_id: String,
    /// Contract-authored stable policy name.
    pub name: String,
    /// Authored canonical Rust type identity.
    pub identity: String,
    /// Exact repository-relative declaration path.
    pub path: String,
    /// Type or authority-token semantic kind.
    pub kind: String,
    /// Source reachability selected by the policy.
    pub reachability: String,
    /// Required linearity mode.
    pub linearity: String,
    /// Independent per-type duplication prohibitions.
    pub deny: Vec<String>,
    /// Expected declaration visibility, when governed.
    pub visibility: Option<String>,
    /// Expected leaf-module state, when governed.
    pub leaf_module: Option<bool>,
    /// Expected exact ordered field representation, when governed.
    pub fields: Option<Vec<GovernedTypeField>>,
    /// Contract-authored justification.
    pub reason: String,
    /// Declaration, derive, impl, and opaque-expansion observations.
    pub observations: Vec<GovernedTypeObservation>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
/// One expected or observed exact Rust field.
pub struct GovernedTypeField {
    /// Exact field name.
    pub name: String,
    /// Complete canonical field type, or an unresolved marker for observations.
    pub type_identity: String,
    /// Semantic Rust visibility.
    pub visibility: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
/// One source fact relevant to an exact Rust type policy.
pub struct GovernedTypeObservation {
    /// Repository-relative source path.
    pub path: String,
    /// Declaration, derive, manual-impl, or opaque-expansion operation.
    pub operation: String,
    /// Analyzer-observed primary identity.
    pub observed: String,
    /// Every canonical candidate retained by the analyzer.
    pub canonical: Vec<String>,
    /// Declaration kind for declaration observations.
    pub declaration_kind: Option<String>,
    /// Declaration visibility for declaration observations.
    pub visibility: Option<String>,
    /// Leaf-module state for declaration observations.
    pub leaf_module: Option<bool>,
    /// Exact ordered observed fields for declaration observations.
    pub fields: Option<Vec<GovernedTypeField>>,
    /// Source coordinates retained by the parser.
    pub span: SourceSpan,
    /// Lexical scopes enclosing the observed source fact.
    pub lexical_scope: Vec<SourceSpan>,
    /// Resolution confidence for this observation.
    pub quality: AnalysisQuality,
    /// Effective syntax guard in kebab-case.
    pub guard: String,
    /// Cargo compilation domains where the observation is available.
    pub compilation_domains: Vec<GovernedCompilationDomain>,
    /// Whether the authored policy permits the observation.
    pub allowed: bool,
    /// Whether exact provenance closes an opaque expansion, when applicable.
    pub closed: Option<bool>,
}
