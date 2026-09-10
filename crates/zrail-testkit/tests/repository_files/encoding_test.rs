//! License equality can preserve a legacy UTF-8 read precondition without normalization.

use std::fs;

use super::support::{configured, coverage, violation};
use zrail_core::ReportStatus;

#[test]
fn utf8_equality_rejects_invalid_bytes_on_either_side_and_preserves_exact_text() {
    let repository = configured(
        r#"
[[repository.files]]
name = "license-copy"
include = ["COPY"]
reason = "License copies retain exact UTF-8 text."
predicate = { kind = "bytes-equal", other = "LICENSE", utf8 = true }
"#,
    );
    let valid = "License ©\r\n".as_bytes();
    for (reference, copy) in [
        (valid, valid),
        (&[0xff][..], &[0xff][..]),
        (valid, &[0xff][..]),
        (&[0xff][..], valid),
        (valid, "License ©\n".as_bytes()),
    ] {
        fs::write(repository.0.join("LICENSE"), reference).expect("reference");
        fs::write(repository.0.join("COPY"), copy).expect("copy");
        let observed = coverage(&repository);
        let policy = &observed.repository_files[0];
        assert_eq!(policy.claim, "utf8-file-bytes");
        assert_eq!(
            policy
                .reference
                .as_ref()
                .expect("reference input")
                .valid_utf8,
            Some(std::str::from_utf8(reference).is_ok())
        );
        assert_eq!(
            policy.entries[0].valid_utf8,
            Some(std::str::from_utf8(copy).is_ok())
        );
        if std::str::from_utf8(reference).is_ok() && reference == copy {
            repository.lock();
            assert_eq!(repository.check().report.status, ReportStatus::Pass);
        } else {
            violation(&repository, "license-copy", "REP-FILE-005");
        }
    }
    assert_eq!(
        repository.explain("COPY").repository_files[0].claim,
        "utf8-file-bytes"
    );
}
