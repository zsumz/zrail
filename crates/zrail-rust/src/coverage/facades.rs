//! Physical written facade coverage shares selection and violations with enforcement.

use serde::Serialize;
use zrail_core::{AnalysisQuality, FacadeMode, SourceSpan};

use crate::{engine::RepositoryModel, inventory::FileClass, source_policy};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
/// Effective facade structure for one physical Rust file, independent of test execution.
pub struct GovernedFacade {
    /// Canonical global or exact-path policy identity.
    pub policy_id: String,
    /// Exact repository-relative path.
    pub path: String,
    /// Inferred source role, unchanged by a test-facade declaration.
    pub source_role: String,
    /// Whether this physical file is reachable only in test compilation.
    pub test_only: bool,
    /// Effective written-item policy; this does not claim expansion execution.
    pub mode: FacadeMode,
    /// Whether the parser context supports this structural mode; false is RUST-FACADE-002.
    pub syntax_allowed: bool,
    /// Exact-path justification, when selected explicitly.
    pub reason: Option<String>,
    /// Every rejected written item in physical source order, without display truncation.
    pub violations: Vec<GovernedFacadeItem>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
/// One written item rejected by the effective facade policy.
pub struct GovernedFacadeItem {
    /// Closed parser item kind, such as struct, function, or non-reexport import.
    pub kind: String,
    /// Physical source coordinates.
    pub span: Option<SourceSpan>,
    /// Syntax observation quality, distinct from semantic or execution evidence.
    pub quality: AnalysisQuality,
}

pub(super) fn report(model: &RepositoryModel) -> Vec<GovernedFacade> {
    let rust = &model.bundle.contract.source.rust;
    let mut seen = std::collections::BTreeSet::new();
    let mut reachability = std::collections::BTreeMap::new();
    for file in &model.source.files {
        let state = reachability
            .entry(&file.relative)
            .or_insert(crate::source::Reachability::UNREACHABLE);
        *state = state.join(file.reachability);
    }
    model
        .source
        .files
        .iter()
        .filter_map(|file| {
            let mode = source_policy::facade_mode_for(&file.relative, file.class, rust)?;
            if !seen.insert(&file.relative) {
                return None;
            }
            let declared = rust
                .file_roles
                .iter()
                .find(|role| role.path == file.relative);
            let policy_id = declared.map_or_else(
                || {
                    if file.class == FileClass::EntryPoint {
                        "rust:entrypoints"
                    } else {
                        "rust:facades"
                    }
                    .into()
                },
                |role| format!("rust:file-role:{}", role.path),
            );
            Some(GovernedFacade {
                policy_id,
                path: file.relative.clone(),
                source_role: source_policy::role_name(file.class).into(),
                test_only: reachability[&file.relative].is_test_only(),
                mode,
                syntax_allowed: crate::rules::facade_syntax_allowed(rust, file, mode),
                reason: declared.map(|role| role.reason.clone()),
                violations: crate::rules::facade_violations(rust, file, mode)
                    .map(|item| GovernedFacadeItem {
                        kind: item.name.clone(),
                        span: item.span,
                        quality: item.quality,
                    })
                    .collect(),
            })
        })
        .collect()
}
