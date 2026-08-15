//! Stable contract-source hashing for lock drift detection.

use sha2::{Digest, Sha256};

use super::load::ContractSource;

pub(super) fn contract_sha256(sources: &[ContractSource]) -> String {
    let mut ordered = sources.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| left.path.cmp(&right.path));
    let mut digest = Sha256::new();
    for source in ordered {
        digest.update(source.path.as_bytes());
        digest.update([0]);
        digest.update(source.content.as_bytes());
        digest.update([0xff]);
    }
    format!("{:x}", digest.finalize())
}
