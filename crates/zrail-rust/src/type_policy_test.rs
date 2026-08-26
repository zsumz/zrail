//! Exact type rails cover authored, derived, imported, and opaque duplication.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use super::check_repository;

static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

#[test]
fn linear_enum_rejects_alias_impl_qualified_impl_derives_and_item_macro() {
    let root = repository(TYPE_POLICY);
    write(
        &root,
        "src/lib.rs",
        r"//! facade
use std::clone::Clone as C;

#[derive(Clone, Copy)]
enum Ticket { One }

impl C for Ticket {
    fn clone(&self) -> Self { Self::One }
}
impl core::marker::Copy for Ticket {}

macro_rules! unrelated { () => { const OTHER: () = (); } }
unrelated!();
",
    );

    let report = report(&root);

    assert_eq!(error_count(&report, "RUST-TYPE-003"), 2, "{report}");
    assert_eq!(error_count(&report, "RUST-TYPE-004"), 2, "{report}");
    assert_eq!(error_count(&report, "RUST-TYPE-005"), 1, "{report}");
    assert_eq!(error_count(&report, "RUST-TYPE-006"), 1, "{report}");
    reset(&root);
}

#[test]
fn authority_token_accepts_recursive_private_exact_shape() {
    let root = repository(AUTHORITY_POLICY);
    write(
        &root,
        "src/lib.rs",
        "//! facade\nstruct Owner;\nmod authority;\n",
    );
    write(
        &root,
        "src/authority.rs",
        "//! authority leaf\nstruct Permit { epoch: u64, owner: core::option::Option<crate::Owner> }\n",
    );

    let report = report(&root);

    assert_eq!(error_count(&report, "RUST-TYPE-001"), 0, "{report}");
    assert_eq!(error_count(&report, "RUST-TYPE-002"), 0, "{report}");
    assert_eq!(error_count(&report, "RUST-TYPE-003"), 0, "{report}");
    assert_eq!(error_count(&report, "RUST-TYPE-004"), 0, "{report}");
    assert_eq!(error_count(&report, "RUST-TYPE-005"), 0, "{report}");
    reset(&root);
}

#[test]
fn authority_token_reports_field_representation_drift() {
    let root = repository(&AUTHORITY_POLICY.replace("type = \"u64\"", "type = \"u128\""));
    write(
        &root,
        "src/lib.rs",
        "//! facade\nstruct Owner;\nmod authority;\n",
    );
    write(
        &root,
        "src/authority.rs",
        "//! authority leaf\nstruct Permit { epoch: u64, owner: core::option::Option<crate::Owner> }\n",
    );

    let report = report(&root);

    assert_eq!(error_count(&report, "RUST-TYPE-002"), 1, "{report}");
    reset(&root);
}

#[test]
fn global_macro_token_policy_is_independent_of_per_type_policy() {
    let root = repository(MACRO_TOKEN_POLICY);
    write(
        &root,
        "src/lib.rs",
        "//! facade\nmacro_rules! mention { () => { (Clone, Copy) } }\n",
    );

    let report = report(&root);

    assert_eq!(error_count(&report, "RUST-TYPE-007"), 2, "{report}");
    reset(&root);
}

#[test]
fn disabled_feature_world_impls_and_macros_do_not_affect_linearity() {
    let root = repository(FEATURE_WORLD_POLICY);
    write(
        &root,
        "Cargo.toml",
        "[package]\nname = \"policy-app\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[features]\ndup = []\n",
    );
    write(
        &root,
        "src/lib.rs",
        r#"//! facade
struct Ticket;

#[cfg(feature = "dup")]
impl Clone for Ticket {
    fn clone(&self) -> Self { Self }
}

#[cfg(feature = "dup")]
macro_rules! duplicate { () => { impl Copy for Ticket {} } }
#[cfg(feature = "dup")]
duplicate!();
"#,
    );

    let report = report(&root);

    assert_eq!(error_count(&report, "RUST-TYPE-004"), 0, "{report}");
    assert_eq!(error_count(&report, "RUST-TYPE-005"), 0, "{report}");
    reset(&root);
}

#[test]
fn unresolved_impl_target_in_the_governed_world_fails_closed() {
    let root = repository(TYPE_POLICY);
    write(
        &root,
        "src/lib.rs",
        r"//! facade
struct Ticket;
use self::U as T;
use self::T as U;

impl Clone for T {
    fn clone(&self) -> Self { unreachable!() }
}
",
    );

    let report = report(&root);

    assert_eq!(error_count(&report, "RUST-TYPE-004"), 1, "{report}");
    reset(&root);
}

pub(crate) fn repository(policy: &str) -> PathBuf {
    let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("zrail-type-policy-{}-{id}", std::process::id()));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture source");
    write(
        &root,
        "Cargo.toml",
        "[package]\nname = \"policy-app\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    );
    write(&root, "zrail.toml", &format!("{BASE_CONTRACT}\n{policy}"));
    root
}

pub(crate) fn report(root: &Path) -> String {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check repository")
        .report
        .human()
}

pub(crate) fn error_count(report: &str, id: &str) -> usize {
    report
        .lines()
        .filter(|line| line.starts_with(&format!("error[{id}]")))
        .count()
}

pub(crate) fn write(root: &Path, path: &str, content: &str) {
    fs::write(root.join(path), content).expect("write fixture");
}

pub(crate) fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const BASE_CONTRACT: &str = r#"schema = 2
adapters = ["rust"]

[repository]
roots = ["."]
exclude = []
workspace_members = "exact"
nested_git = "deny"
submodules = "deny"
symlinks = "inside"

[dependencies]
mode = "observed"
unassigned_packages = "allow"
cycles = "allow"

[source.rust]
module_docs = "allow"
facades = "allow"
tests = "allow"

[source.rust.hygiene]
unsafe = "allow"
lint_suppressions = "allow"
"#;

pub(crate) const TYPE_POLICY: &str = r#"[source.rust.duplication]
deny_imports = ["clone"]

[[source.rust.types]]
name = "ticket-linearity"
match = "crate::Ticket"
path = "src/lib.rs"
kind = "type"
linearity = "required"
reason = "Tickets must transfer rather than duplicate."
"#;

const AUTHORITY_POLICY: &str = r#"[[source.rust.types]]
name = "permit-authority"
match = "crate::authority::Permit"
path = "src/authority.rs"
kind = "authority-token"
linearity = "required"
visibility = "private"
leaf_module = true
reason = "Permits carry one-use authority."

[[source.rust.types.fields]]
name = "epoch"
type = "u64"
visibility = "private"

[[source.rust.types.fields]]
name = "owner"
type = "core::option::Option<crate::Owner>"
visibility = "private"
"#;

const MACRO_TOKEN_POLICY: &str = r#"[source.rust.duplication]
deny_macro_tokens = ["clone", "copy"]
"#;

pub(crate) const FEATURE_WORLD_POLICY: &str = r#"[[source.rust.feature_worlds]]
name = "without-dup"
reason = "The supported build excludes optional duplication syntax."

[[source.rust.feature_worlds.packages]]
package = "policy-app"
default_features = false
features = []

[[source.rust.types]]
name = "ticket-linearity"
match = "crate::Ticket"
path = "src/lib.rs"
kind = "type"
linearity = "required"
reason = "Tickets remain linear in every configured world."
"#;
