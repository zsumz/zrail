//! Explicit reviewed Rust fragments extend discovery without inferring source authority from names.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::Path,
};

use serde::Deserialize;

use super::snapshot::FileRecord;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AdditionalInputs {
    schema: u32,
    inputs: Vec<Input>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    repository: String,
    commit: String,
    path: String,
    sha256: String,
    syntax: Syntax,
    reason: String,
    review_sources: Vec<ReviewSource>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Syntax {
    Items,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewSource {
    path: String,
    sha256: String,
}

impl AdditionalInputs {
    pub(super) fn read(directory: &Path) -> (Self, String) {
        let mut bytes = Vec::new();
        fs::File::open(directory.join("additional-rust-inputs.json"))
            .expect("reviewed fragment inputs")
            .take(1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .expect("bounded fragment registry");
        (Self::parse(&bytes), zrail_core::sha256_hex(&bytes))
    }

    pub(super) fn parse(bytes: &[u8]) -> Self {
        assert!(
            bytes.len() <= 1024 * 1024,
            "fragment registry exceeds 1 MiB"
        );
        let inputs: Self = serde_json::from_slice(bytes).expect("strict fragment registry");
        assert_eq!(inputs.schema, 1, "fragment registry schema");
        assert!(inputs.inputs.len() <= 64, "bounded fragment registry");
        let mut identities = BTreeSet::new();
        for input in &inputs.inputs {
            assert!(
                identities.insert((&input.repository, &input.path)),
                "duplicate fragment input"
            );
            assert!(!input.repository.is_empty() && !input.reason.trim().is_empty());
            assert!(digest(&input.commit, 40) && digest(&input.sha256, 64));
            assert!(
                path(&input.path)
                    && Path::new(&input.path).extension() != Some(std::ffi::OsStr::new("rs")),
                "exact additional Rust path"
            );
            assert!(!input.review_sources.is_empty() && input.review_sources.len() <= 32);
            let mut sources = BTreeSet::new();
            for source in &input.review_sources {
                assert!(sources.insert(&source.path), "duplicate review source");
                assert!(path(&source.path) && digest(&source.sha256, 64));
            }
        }
        inputs
    }

    pub(super) fn selected(
        &self,
        repository: &str,
        commit: &str,
        path: &str,
        sha256: &str,
    ) -> bool {
        self.inputs.iter().any(|input| {
            if input.repository != repository || input.path != path {
                return false;
            }
            assert_eq!(input.commit, commit, "fragment source snapshot");
            assert_eq!(input.sha256, sha256, "fragment bytes");
            match input.syntax {
                Syntax::Items => true,
            }
        })
    }

    pub(super) fn verify(&self, files: &[FileRecord]) {
        let files = files
            .iter()
            .map(|file| ((file.repository.as_str(), file.path.as_str()), file))
            .collect::<BTreeMap<_, _>>();
        for input in &self.inputs {
            let selected = files
                .get(&(input.repository.as_str(), input.path.as_str()))
                .expect("tracked fragment input");
            assert!(matches!(selected.mode.as_str(), "100644" | "100755"));
            assert_eq!(selected.commit, input.commit);
            assert_eq!(selected.sha256.as_deref(), Some(input.sha256.as_str()));
            for source in &input.review_sources {
                let reviewed = files
                    .get(&(input.repository.as_str(), source.path.as_str()))
                    .expect("tracked review source");
                assert_eq!(reviewed.commit, input.commit);
                assert_eq!(reviewed.sha256.as_deref(), Some(source.sha256.as_str()));
            }
        }
    }
}

fn path(value: &str) -> bool {
    !value.contains(['\\', '\0', ':', '*', '?'])
        && value
            .split('/')
            .all(|part| !matches!(part, "" | "." | ".."))
}

fn digest(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
