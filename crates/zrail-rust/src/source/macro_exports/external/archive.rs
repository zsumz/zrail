//! Cargo cache archives are accepted only after Cargo.lock checksum verification.

#[path = "archive/manifest.rs"]
mod manifest;
#[path = "archive/paths.rs"]
mod paths;

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

use flate2::read::GzDecoder;

use crate::cargo::ResolvedPackageIdentity;
use paths::checked_member_path;

pub(super) use paths::normalized_relative;

const MAX_ARCHIVE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_EXPANDED_BYTES: u64 = 128 * 1024 * 1024;
const MAX_ARCHIVE_STREAM_BYTES: u64 = 160 * 1024 * 1024;
const MAX_SOURCE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_MEMBERS: usize = 16_384;
const MAX_CACHE_DIRECTORIES: usize = 256;

#[derive(Debug)]
pub(super) struct VerifiedPackage {
    pub(super) files: BTreeMap<String, String>,
    pub(super) library: String,
}

pub(super) fn load(identity: &ResolvedPackageIdentity) -> Result<VerifiedPackage, String> {
    if !identity.source.starts_with("registry+") {
        return Err(format!(
            "external macro source {} is not a checksum-bound registry package",
            identity.label()
        ));
    }
    let checksum = identity.checksum.as_deref().ok_or_else(|| {
        format!(
            "registry package {} has no Cargo.lock checksum",
            identity.label()
        )
    })?;
    if checksum.len() != 64 || !checksum.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!(
            "registry package {} has an invalid Cargo.lock checksum",
            identity.label()
        ));
    }
    let archive_name = archive_name(identity)?;
    let candidates = cache_candidates(&archive_name)?;
    let mut mismatches = 0usize;
    for candidate in candidates {
        let metadata = fs::symlink_metadata(&candidate)
            .map_err(|error| format!("cannot inspect Cargo cache archive: {error}"))?;
        if !metadata.file_type().is_file() || metadata.len() > MAX_ARCHIVE_BYTES {
            continue;
        }
        let bytes = fs::read(&candidate)
            .map_err(|error| format!("cannot read Cargo cache archive: {error}"))?;
        if zrail_core::sha256_hex(&bytes) != checksum.to_ascii_lowercase() {
            mismatches += 1;
            continue;
        }
        return unpack(identity, &bytes);
    }
    if mismatches > 0 {
        Err(format!(
            "Cargo cache archive {archive_name:?} does not match its Cargo.lock checksum"
        ))
    } else {
        Err(format!(
            "Cargo cache archive {archive_name:?} is unavailable for offline macro export analysis"
        ))
    }
}

fn archive_name(identity: &ResolvedPackageIdentity) -> Result<String, String> {
    for value in [&identity.name, &identity.version] {
        if value.is_empty()
            || !value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'+')
            })
        {
            return Err(format!(
                "Cargo.lock package archive component {value:?} is invalid"
            ));
        }
    }
    Ok(format!("{}-{}.crate", identity.name, identity.version))
}

fn cache_candidates(archive_name: &str) -> Result<Vec<PathBuf>, String> {
    let home = cargo_home().ok_or_else(|| "Cargo home is unavailable".to_owned())?;
    let cache = home.join("registry/cache");
    let mut directories = fs::read_dir(&cache)
        .map_err(|error| format!("cannot inspect Cargo registry cache: {error}"))?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    directories.sort();
    if directories.len() > MAX_CACHE_DIRECTORIES {
        return Err("Cargo registry cache directory count exceeds the analysis limit".into());
    }
    Ok(directories
        .into_iter()
        .map(|directory| directory.join(archive_name))
        .filter(|path| path.exists())
        .collect())
}

fn cargo_home() -> Option<PathBuf> {
    std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .map(|home| PathBuf::from(home).join(".cargo"))
        })
}

fn unpack(identity: &ResolvedPackageIdentity, bytes: &[u8]) -> Result<VerifiedPackage, String> {
    let expected_root = format!("{}-{}", identity.name, identity.version);
    let decoder = GzDecoder::new(bytes).take(MAX_ARCHIVE_STREAM_BYTES + 1);
    let mut archive = tar::Archive::new(decoder);
    let entries = archive
        .entries()
        .map_err(|error| format!("cannot read checksum-matched crate archive: {error}"))?;
    let mut files = BTreeMap::new();
    let mut seen = BTreeSet::new();
    let mut total = 0u64;
    let mut count = 0usize;
    for entry in entries {
        count += 1;
        if count > MAX_MEMBERS {
            return Err("crate archive member count exceeds the analysis limit".into());
        }
        let mut entry = entry
            .map_err(|error| format!("cannot read checksum-matched crate archive: {error}"))?;
        let relative = checked_member_path(
            &entry.path().map_err(|error| {
                format!("crate archive contains an invalid member path: {error}")
            })?,
            &expected_root,
        )?;
        if !seen.insert(relative.clone()) {
            return Err(format!("crate archive repeats member {relative:?}"));
        }
        let kind = entry.header().entry_type();
        if kind.is_dir() {
            continue;
        }
        if !kind.is_file() {
            return Err(format!(
                "crate archive member {relative:?} is not a regular file"
            ));
        }
        let size = entry.size();
        total = total
            .checked_add(size)
            .filter(|total| *total <= MAX_EXPANDED_BYTES)
            .ok_or_else(|| "crate archive expanded size exceeds the analysis limit".to_owned())?;
        let rust_source = Path::new(&relative)
            .extension()
            .is_some_and(|extension| extension == "rs");
        if relative == "Cargo.toml" || rust_source {
            if size > MAX_SOURCE_BYTES {
                return Err(format!(
                    "crate source member {relative:?} exceeds the analysis limit"
                ));
            }
            let mut source = Vec::with_capacity(usize::try_from(size).unwrap_or(0));
            entry.read_to_end(&mut source).map_err(|error| {
                format!("cannot read crate source member {relative:?}: {error}")
            })?;
            if source.len() as u64 != size {
                return Err(format!(
                    "crate source member {relative:?} has an invalid size"
                ));
            }
            let source = String::from_utf8(source)
                .map_err(|_| format!("crate source member {relative:?} is not UTF-8"))?;
            files.insert(relative, source);
        } else {
            let copied = io::copy(&mut entry, &mut io::sink())
                .map_err(|error| format!("cannot read crate archive member: {error}"))?;
            if copied != size {
                return Err("crate archive member has an invalid size".into());
            }
        }
    }
    let mut decoder = archive.into_inner();
    io::copy(&mut decoder, &mut io::sink())
        .map_err(|error| format!("cannot finish checksum-matched crate archive: {error}"))?;
    if decoder.limit() == 0 {
        return Err("crate archive stream exceeds the analysis limit".into());
    }
    manifest::read(identity, files)
}

#[cfg(test)]
#[path = "archive_test.rs"]
mod archive_test;
