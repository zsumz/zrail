//! Borrowed identities avoid cloning complete lock nodes during set comparison.

use serde::Serialize;
use zrail_core::LockPackageIdentity;

use crate::cargo::ResolvedPackageIdentity;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(super) struct Identity<'a> {
    version: &'a str,
    source: &'a str,
    checksum: Option<&'a str>,
}

impl<'a> From<&'a ResolvedPackageIdentity> for Identity<'a> {
    fn from(value: &'a ResolvedPackageIdentity) -> Self {
        Self {
            version: &value.version,
            source: &value.source,
            checksum: value.checksum.as_deref(),
        }
    }
}

impl<'a> From<&'a LockPackageIdentity> for Identity<'a> {
    fn from(value: &'a LockPackageIdentity) -> Self {
        Self {
            version: &value.version,
            source: &value.source,
            checksum: value.checksum.as_deref(),
        }
    }
}

impl Identity<'_> {
    pub(super) fn work(self) -> usize {
        1 + self.version.len() + self.source.len() + self.checksum.map_or(0, str::len)
    }

    pub(super) fn owned(self) -> LockPackageIdentity {
        LockPackageIdentity {
            version: self.version.into(),
            source: self.source.into(),
            checksum: self.checksum.map(str::to_owned),
        }
    }
}

pub(super) fn sample<'a>(
    identities: impl Iterator<Item = Identity<'a>>,
) -> Vec<LockPackageIdentity> {
    let mut selected = Vec::new();
    let mut bytes = 2;
    for identity in identities {
        if selected.len() == 16 {
            break;
        }
        if identity.work() > 16 * 1_024 {
            continue;
        }
        // Serialization can expand escaped source/version text; raw length is only a precheck.
        let Ok(encoded) = serde_json::to_vec(&identity) else {
            continue;
        };
        let next = bytes + encoded.len() + usize::from(!selected.is_empty());
        if next <= 16 * 1_024 {
            selected.push(identity.owned());
            bytes = next;
        }
    }
    selected
}
