//! Source-syntax policy coverage records.

use serde::Serialize;
use zrail_core::{AnalysisQuality, SourceSpan};

use super::GovernedCompilationDomain;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
/// One configured source policy and every syntax occurrence in its boundary.
pub struct GovernedSourcePolicyRail {
    /// Canonical report identity for this policy.
    pub policy_id: String,
    /// Closed policy mode or denied syntax identity.
    pub policy: String,
    /// Profile applying the policy, when it is profile-scoped.
    pub profile: Option<String>,
    /// Source reachability selected by the policy.
    pub reachability: String,
    /// Exact occurrences in deterministic source order.
    pub occurrences: Vec<GovernedSourcePolicyOccurrence>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
/// One written source occurrence selected by a syntax or import policy.
pub struct GovernedSourcePolicyOccurrence {
    /// Repository-relative source path.
    pub path: String,
    /// Kind of syntax represented by this occurrence.
    pub operation: String,
    /// Exact written target or syntax identity.
    pub observed: String,
    /// Visibility of an import occurrence, when applicable.
    pub visibility: Option<String>,
    /// Lexical scopes enclosing the syntax occurrence.
    pub lexical_scope: Vec<SourceSpan>,
    /// Source coordinates retained by the parser.
    pub span: SourceSpan,
    /// Resolution confidence for the occurrence.
    pub quality: AnalysisQuality,
    /// Effective syntax guard in kebab-case.
    pub guard: String,
    /// Cargo compilation domains where the guarded occurrence is available.
    pub compilation_domains: Vec<GovernedCompilationDomain>,
    /// Whether the configured policy permits this exact occurrence.
    pub allowed: bool,
}
