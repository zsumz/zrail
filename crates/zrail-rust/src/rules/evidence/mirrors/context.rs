//! Selected Cargo-package ownership for an attested test invocation.

use zrail_core::{FindingSink, TestMirrorContract};

use crate::{rules::RuleContext, source::RustFileFacts};

use super::findings::mirror_finding;

pub(super) fn check(
    mirror: &TestMirrorContract,
    production: Option<&RustFileFacts>,
    test: Option<&RustFileFacts>,
    context: &RuleContext<'_>,
    findings: &mut FindingSink,
) {
    let matches = context
        .cargo
        .packages
        .iter()
        .filter(|package| package.name == mirror.execution.package)
        .collect::<Vec<_>>();
    let [package] = matches.as_slice() else {
        findings.push(mirror_finding(
            "MIRROR-007",
            mirror,
            &mirror.production,
            "execution package does not select exactly one workspace package",
        ));
        return;
    };
    if production.is_some_and(|file| !file.packages.contains(&package.name))
        || test.is_some_and(|file| !file.packages.contains(&package.name))
    {
        findings.push(mirror_finding(
            "MIRROR-008",
            mirror,
            &mirror.production,
            "execution package does not own both exact mirror sources",
        ));
    }
    let manifest = package.manifest_path();
    if !mirror.inputs.contains(&manifest) {
        findings.push(mirror_finding(
            "MIRROR-009",
            mirror,
            &manifest,
            "selected package manifest is absent from reviewed mirror inputs",
        ));
    }
}
