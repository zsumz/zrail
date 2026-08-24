//! Mirror diagnostics retain their exact governed source and receipt paths.

use zrail_core::{Finding, TestMirrorContract};

pub(super) fn mirror_finding(
    id: &str,
    mirror: &TestMirrorContract,
    path: &str,
    message: &str,
) -> Finding {
    Finding::error(id, "rust.test-mirror", &mirror.production, message).at(path, None)
}

pub(super) fn receipt_finding(id: &str, mirror: &TestMirrorContract, message: &str) -> Finding {
    Finding::error(id, "rust.test-mirror-receipt", &mirror.production, message)
        .at(&mirror.receipt, None)
}
