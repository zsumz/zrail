//! The prefix proof retains all five frozen identities and complete original execution.

pub(super) use super::super::super::model as lock_model;
use std::{
    fs,
    path::{Path, PathBuf},
};
use zrail_core::{LockPackageAssertion, LockPackageIdentity, LockPackageRule, sha256_hex};

pub(super) use lock_model::project;
pub(super) const POLICY: &str = "docs/rc9/policies/kafka-driver.lock-packages.fragment.toml";
pub(super) const ORIGINS: &str = "crates/zrail-testkit/tests/fixtures/rc9/lock-prefix-origins.json";
pub(super) const CASES: &str = "crates/zrail-testkit/tests/fixtures/rc9/lock-prefix-cases.json";
pub(super) const SOURCE_SHA: &str =
    "490a52269cd18516e625db062b0578c1e1f8c7208d57e7f9228d33f19de43573";

pub(super) fn identity(rule: &LockPackageRule) -> &LockPackageIdentity {
    let LockPackageAssertion::Exact { identities } = &rule.assertion else {
        panic!("exact rule");
    };
    assert_eq!(identities.len(), 1);
    &identities[0]
}

pub(super) fn candidate(rule: &LockPackageRule, version: &str) -> LockPackageRule {
    let mut rule = rule.clone();
    let LockPackageAssertion::Exact { identities } = &mut rule.assertion else {
        panic!("exact rule");
    };
    identities[0].version = version.into();
    rule
}

pub(super) fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|text| (*text).into()))
        .expect("original panic")
}

pub(super) fn source_binding() {
    let registry: serde_json::Value =
        serde_json::from_slice(&fs::read(project().join(ORIGINS)).expect("origins")).expect("JSON");
    let source =
        fs::read_to_string(project().join("crates/zrail-rust/tests/rc9_lock_prefix/original.rs"))
            .expect("excerpt");
    let start = source
        .find("    let version = version")
        .expect("prefix body");
    let excerpt = source[start..]
        .split_inclusive('\n')
        .take(3)
        .collect::<String>();
    assert_eq!(
        sha256_hex(excerpt.as_bytes()),
        registry["extractions"][0]["extracted_sha256"]
            .as_str()
            .expect("digest")
    );
}

pub(super) fn original_helper(
    lock: &toml::Value,
    rule: &LockPackageRule,
    source_version: &str,
) -> Result<(), String> {
    std::panic::catch_unwind(|| {
        super::super::assert_locked(
            lock,
            &rule.package,
            source_version,
            identity(rule).checksum.as_deref().expect("checksum"),
        );
    })
    .map_err(|payload| panic_text(payload.as_ref()))
}

pub(super) struct Fixture(pub(super) PathBuf);
impl Fixture {
    pub(super) fn new() -> Self {
        let path = std::env::temp_dir()
            .canonicalize()
            .expect("canonical temp")
            .join(format!(
                "zrail-lock-prefix-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
        assert!(!path.exists(), "fresh test-owned fixture");
        fs::create_dir(&path).expect("create fixture");
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove only test-owned fixture");
    }
}

pub(super) fn frozen(root: &Path) -> Vec<crate::GovernedLockPackage> {
    super::super::check(root);
    let workspace = lock_model::workspace(root);
    let (rules, _) = lock_model::policies();
    let observed = lock_model::native(root, &workspace, &rules).expect("complete native graph");
    assert_eq!(observed.len(), 5);
    assert!(
        observed
            .iter()
            .all(|row| row.satisfied && row.observed_count == 1)
    );
    observed
}
