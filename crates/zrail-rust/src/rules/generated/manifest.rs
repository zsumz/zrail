//! Bounded decoding for the generic checked-in generator manifest.

use std::path::Path;

use serde::Deserialize;
use zrail_core::{GeneratedSourceContract, normalize_relative, read_text_with_limit};

pub(super) const MAX_FILES: usize = 20_000;
pub(super) const MAX_INPUTS: usize = 20_000;
pub(super) const MAX_SOURCE_BYTES: usize = 2 * 1024 * 1024;
pub(super) const MAX_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Deserialize)]
pub(super) struct Manifest {
    pub(super) schema: u64,
    pub(super) generator: String,
    pub(super) inputs: Vec<ManifestInput>,
    pub(super) files: Vec<ManifestFile>,
}

#[derive(Debug, Deserialize)]
pub(super) struct ManifestInput {
    pub(super) path: String,
    pub(super) sha256: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct ManifestFile {
    pub(super) path: String,
    pub(super) sha256: String,
}

pub(super) fn read(
    repository: &Path,
    generated: &GeneratedSourceContract,
) -> Result<Manifest, String> {
    let source = read_text_with_limit(&repository.join(&generated.manifest), MAX_BYTES)?;
    serde_json::from_str(&source)
        .map_err(|error| format!("generated manifest is invalid JSON: {error}"))
}

pub(super) fn file_path(root: &str, path: &str) -> Result<String, String> {
    let normalized = normalize_relative(Path::new(path))
        .map_err(|error| format!("generated manifest path {path:?} is invalid: {error}"))?;
    if normalized.is_empty() || normalized != path || banner(path).is_none() {
        return Err(format!(
            "generated manifest path must be a normalized .rs or .rsi path: {path:?}"
        ));
    }
    Ok(if root == "." {
        normalized
    } else {
        format!("{root}/{normalized}")
    })
}

pub(super) fn input_path(path: &str) -> Result<String, String> {
    let normalized = normalize_relative(Path::new(path))
        .map_err(|error| format!("generated input path {path:?} is invalid: {error}"))?;
    if normalized.is_empty() || normalized != path {
        return Err(format!(
            "generated input path must be a normalized repository-relative path: {path:?}"
        ));
    }
    Ok(normalized)
}

pub(super) fn banner(path: &str) -> Option<&'static str> {
    match Path::new(path).extension().and_then(|value| value.to_str()) {
        Some("rs") => Some("//! @generated"),
        Some("rsi") => Some("// @generated"),
        _ => None,
    }
}

pub(super) fn source_candidate(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("rs") || extension.eq_ignore_ascii_case("rsi")
        })
}
