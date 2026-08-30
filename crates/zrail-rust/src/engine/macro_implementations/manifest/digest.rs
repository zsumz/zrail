//! Bounded deterministic framing of captured implementation paths and bytes.

use std::collections::BTreeMap;

use super::CheckError;

pub(crate) const MAX_IMPLEMENTATION_INPUTS: usize = 4_096;
pub(super) const MAX_IMPLEMENTATION_BYTES: usize = 64 * 1024 * 1024;

pub(crate) fn digest_inputs(inputs: &BTreeMap<String, Vec<u8>>) -> Result<String, CheckError> {
    if inputs.len() > MAX_IMPLEMENTATION_INPUTS {
        return Err(CheckError::from_message(format!(
            "macro implementation exceeds the {MAX_IMPLEMENTATION_INPUTS}-input safety limit"
        )));
    }
    let total = inputs.iter().try_fold(0_usize, |total, (path, bytes)| {
        total
            .checked_add(path.len())
            .and_then(|value| value.checked_add(bytes.len()))
            .and_then(|value| value.checked_add(16))
    });
    if total.is_none_or(|total| total > MAX_IMPLEMENTATION_BYTES) {
        return Err(CheckError::from_message(format!(
            "macro implementation exceeds the {MAX_IMPLEMENTATION_BYTES}-byte safety limit"
        )));
    }
    let mut manifest = Vec::with_capacity(total.unwrap_or_default());
    for (path, bytes) in inputs {
        frame(&mut manifest, path.as_bytes());
        frame(&mut manifest, bytes);
    }
    Ok(zrail_core::sha256_hex(&manifest))
}

fn frame(output: &mut Vec<u8>, bytes: &[u8]) {
    output.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    output.extend_from_slice(bytes);
}
