//! Shared content hashing without adapter-level dependency duplication.

use sha2::{Digest as _, Sha256};

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "digest_test.rs"]
mod digest_test;
