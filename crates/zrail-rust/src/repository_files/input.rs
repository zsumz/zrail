//! Shared bounded file bytes, hashed once and reused across raw predicates.

use std::{collections::BTreeMap, path::Path, rc::Rc};

use zrail_core::{read_bytes_with_limit, repository_relative, sha256_hex};

use super::{boundary, model::GovernedRepositoryFileEntry};

const MAX_FILE_BYTES: usize = 2 * 1024 * 1024;
const MAX_INPUT_BYTES: usize = 64 * 1024 * 1024;
const MAX_CONTENT_WORK: usize = 256 * 1024 * 1024;

#[derive(Clone)]
struct Input {
    bytes: Rc<[u8]>,
    sha256: String,
}

#[derive(Default)]
pub(super) struct Inputs {
    cache: BTreeMap<String, Input>,
    bytes: usize,
    work: usize,
}

impl Inputs {
    pub(super) fn read(
        &mut self,
        root: &Path,
        entry: &mut GovernedRepositoryFileEntry,
    ) -> Result<Rc<[u8]>, String> {
        let written = entry.resolved_path.as_deref().unwrap_or(&entry.path);
        let path = boundary::contained(root, &root.join(written))?;
        let relative = repository_relative(root, &path)?;
        if !self.cache.contains_key(&relative) {
            let bytes = read_bytes_with_limit(&path, MAX_FILE_BYTES)?;
            self.bytes += bytes.len();
            if self.bytes > MAX_INPUT_BYTES {
                return Err(format!(
                    "file policies exceed the {MAX_INPUT_BYTES}-byte unique input limit"
                ));
            }
            self.cache.insert(
                relative.clone(),
                Input {
                    sha256: sha256_hex(&bytes),
                    bytes: bytes.into(),
                },
            );
        }
        let input = &self.cache[&relative];
        self.work += input.bytes.len();
        if self.work > MAX_CONTENT_WORK {
            return Err(format!(
                "file policies exceed the {MAX_CONTENT_WORK}-byte content work limit"
            ));
        }
        entry.sha256 = Some(input.sha256.clone());
        entry.bytes = Some(input.bytes.len());
        if relative != entry.path {
            entry.resolved_path = Some(relative);
        }
        Ok(Rc::clone(&input.bytes))
    }
}
