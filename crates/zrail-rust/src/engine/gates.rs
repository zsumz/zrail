//! Bounded content manifests for qualification gates and their effective inputs.

use std::collections::BTreeMap;

use zrail_core::{LockedGate, LockedGateInput, MAX_INPUT_BYTES, read_bytes_with_limit, sha256_hex};

use crate::inventory::{RepositoryEntry, RepositoryEntryKind};

use super::{CheckError, model::RepositoryModel};

const MAX_GATE_BYTES: usize = 64 * 1024 * 1024;

pub(super) fn locked(model: &RepositoryModel) -> Result<Vec<LockedGate>, CheckError> {
    let entries = model
        .inventory
        .entries
        .iter()
        .map(|entry| (entry.relative.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut digests = GateDigests::default();
    let mut gates = Vec::new();
    for gate in &model.bundle.contract.gates {
        let Some(sha256) = digests.digest(&entries, &gate.path)? else {
            continue;
        };
        let mut inputs = Vec::new();
        for path in &gate.inputs {
            if let Some(sha256) = digests.digest(&entries, path)? {
                inputs.push(LockedGateInput {
                    path: path.clone(),
                    sha256,
                });
            }
        }
        gates.push(LockedGate {
            name: gate.name.clone(),
            path: gate.path.clone(),
            sha256,
            inputs,
        });
    }
    Ok(gates)
}

#[derive(Default)]
struct GateDigests {
    bytes: usize,
    values: BTreeMap<String, String>,
}

impl GateDigests {
    fn digest(
        &mut self,
        entries: &BTreeMap<&str, &RepositoryEntry>,
        path: &str,
    ) -> Result<Option<String>, CheckError> {
        if let Some(value) = self.values.get(path) {
            return Ok(Some(value.clone()));
        }
        let Some(entry) = entries.get(path) else {
            return Ok(None);
        };
        if entry.kind != RepositoryEntryKind::File {
            return Ok(None);
        }
        let bytes = read_bytes_with_limit(&entry.absolute, MAX_INPUT_BYTES)
            .map_err(CheckError::from_message)?;
        self.bytes = self.bytes.checked_add(bytes.len()).ok_or_else(|| {
            CheckError::from_message("qualification gate input byte count overflowed")
        })?;
        if self.bytes > MAX_GATE_BYTES {
            return Err(CheckError::from_message(format!(
                "qualification gates exceed {MAX_GATE_BYTES} total input bytes"
            )));
        }
        let value = sha256_hex(&bytes);
        self.values.insert(path.into(), value.clone());
        Ok(Some(value))
    }
}
