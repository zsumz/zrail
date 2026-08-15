//! Strict identities for evidence attached to enforced invariants.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceReference<'a> {
    RustTest { path: &'a str, test: &'a str },
    Gate { name: &'a str },
}

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
