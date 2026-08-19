//! Strict identities for evidence attached to enforced invariants.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// A validated reference to repository-local invariant evidence.
pub enum EvidenceReference<'a> {
    /// A named Rust test in a repository-relative source file.
    RustTest {
        /// Repository-relative Rust source path.
        path: &'a str,
        /// Rust test function identifier without a module qualification.
        test: &'a str,
    },
    /// A named qualification gate declared by the contract.
    Gate {
        /// Gate name without the `gate:` prefix.
        name: &'a str,
    },
}

/// Parses a strict `rust-test:path::test` or `gate:name` evidence identity.
///
/// Returns an error when the prefix is unsupported, the path or name is empty,
/// a Rust test name is not an identifier, or a gate name contains characters
/// other than ASCII letters, digits, `-`, `_`, and `.`.
pub fn parse_evidence_reference(value: &str) -> Result<EvidenceReference<'_>, String> {
    if let Some(identity) = value.strip_prefix("rust-test:") {
        let Some((path, test)) = identity.rsplit_once("::") else {
            return Err(format!(
                "Rust test evidence must be rust-test:path::test, found {value:?}"
            ));
        };
        if path.is_empty() || path.contains("::") || !valid_identifier(test) {
            return Err(format!("invalid Rust test evidence {value:?}"));
        }
        return Ok(EvidenceReference::RustTest { path, test });
    }
    if let Some(name) = value.strip_prefix("gate:") {
        if !valid_name(name) {
            return Err(format!("invalid gate evidence {value:?}"));
        }
        return Ok(EvidenceReference::Gate { name });
    }
    Err(format!(
        "unsupported evidence {value:?}; expected rust-test:path::test or gate:name"
    ))
}

pub(super) fn valid_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

#[cfg(test)]
#[path = "evidence_test.rs"]
mod evidence_test;
