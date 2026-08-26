//! Bounded reading and hashing of exact mirror execution inputs.

use zrail_core::{FindingSink, TestMirrorContract};

use crate::{mirror_inputs::MirrorInputCache, source::RustFileFacts};

use super::findings::receipt_finding;

pub(super) fn digest(
    mirror: &TestMirrorContract,
    production: Option<&RustFileFacts>,
    test: Option<&RustFileFacts>,
    cache: &mut MirrorInputCache<'_>,
    findings: &mut FindingSink,
) -> Option<String> {
    production?;
    test?;
    match cache.digest(mirror) {
        Ok(digest) => Some(digest),
        Err(message) => {
            findings.push(receipt_finding("RECEIPT-006", mirror, &message));
            None
        }
    }
}
