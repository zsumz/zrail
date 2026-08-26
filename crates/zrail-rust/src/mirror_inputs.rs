//! Bounded mirror-input bytes are shared without weakening per-mirror digests.

use std::collections::BTreeMap;

use zrail_core::{
    MAX_INPUT_BYTES, MAX_TEST_MIRROR_INPUT_BYTES, TestMirrorContract, read_bytes_with_limit,
    test_mirror_input_sha256,
};

use crate::inventory::{RepositoryEntry, RepositoryEntryKind};

pub(crate) struct MirrorInputCache<'a> {
    entries: &'a BTreeMap<&'a str, &'a RepositoryEntry>,
    bytes: BTreeMap<String, Vec<u8>>,
    aggregate_bytes: usize,
}

impl<'a> MirrorInputCache<'a> {
    pub(crate) fn new(entries: &'a BTreeMap<&'a str, &'a RepositoryEntry>) -> Self {
        Self {
            entries,
            bytes: BTreeMap::new(),
            aggregate_bytes: 0,
        }
    }

    pub(crate) fn digest(&mut self, mirror: &TestMirrorContract) -> Result<String, String> {
        self.ensure(&mirror.production)?;
        self.ensure(&mirror.test)?;
        for path in &mirror.inputs {
            self.ensure(path)?;
        }
        let production = self.get(&mirror.production)?;
        let test = self.get(&mirror.test)?;
        let reviewed = mirror
            .inputs
            .iter()
            .map(|path| self.get(path).map(|bytes| (path.as_str(), bytes)))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(test_mirror_input_sha256(
            mirror, production, test, &reviewed,
        ))
    }

    fn ensure(&mut self, path: &str) -> Result<(), String> {
        if self.bytes.contains_key(path) {
            return Ok(());
        }
        let entry = self
            .entries
            .get(path)
            .copied()
            .filter(|entry| entry.kind == RepositoryEntryKind::File)
            .ok_or_else(|| format!("mirror input {path:?} is missing or not a regular file"))?;
        let bytes = read_bytes_with_limit(&entry.absolute, MAX_INPUT_BYTES)?;
        let aggregate_bytes = self
            .aggregate_bytes
            .checked_add(bytes.len())
            .ok_or_else(|| "test mirror input byte count overflowed".to_owned())?;
        if aggregate_bytes > MAX_TEST_MIRROR_INPUT_BYTES {
            return Err(format!(
                "test mirror inputs exceed {MAX_TEST_MIRROR_INPUT_BYTES} unique bytes"
            ));
        }
        self.aggregate_bytes = aggregate_bytes;
        self.bytes.insert(path.to_owned(), bytes);
        Ok(())
    }

    fn get(&self, path: &str) -> Result<&[u8], String> {
        self.bytes
            .get(path)
            .map(Vec::as_slice)
            .ok_or_else(|| format!("mirror input {path:?} was not loaded"))
    }

    #[cfg(test)]
    pub(crate) const fn aggregate_bytes(&self) -> usize {
        self.aggregate_bytes
    }

    #[cfg(test)]
    pub(crate) fn loaded_inputs(&self) -> usize {
        self.bytes.len()
    }
}

#[cfg(test)]
#[path = "mirror_inputs_test.rs"]
mod mirror_inputs_test;
