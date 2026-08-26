//! Projected Rust places cannot bypass backing-field ownership.

use std::{fs, path::PathBuf};

use zrail_core::Report;
use zrail_rust::check_repository;

#[test]
fn field_authority_covers_projected_writes_and_mutable_borrows() {
    let root = repository();
    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check projected-place fixture")
        .report;

    for (rule, count) in [
        ("values-authority", 3),
        ("outer-authority", 1),
        ("pointer-authority", 1),
        ("tuple-authority", 1),
        ("index-read", 3),
    ] {
        assert_eq!(
            trespasses(&report, rule),
            count,
            "{rule}: {}",
            report.human()
        );
    }
    reset(&root);
}

fn repository() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-place-ownership-{}-{:?}",
        std::process::id(),
        std::thread::current().id(),
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture source");
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "zrail.toml", CONTRACT);
    write(&root, "src/lib.rs", LIBRARY);
    write(&root, "src/owner.rs", OWNER);
    write(&root, "src/trespasser.rs", TRESPASSER);
    root
}

fn trespasses(report: &Report, rule: &str) -> usize {
    report
        .findings
        .iter()
        .filter(|finding| {
            finding.rule == rule
                && finding.id == "OWN-003"
                && finding.path.as_deref() == Some("src/trespasser.rs")
        })
        .count()
}

fn write(root: &std::path::Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture");
}

fn reset(root: &PathBuf) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const MANIFEST: &str = r#"[package]
name = "place-ownership-fixture"
version = "0.0.0"
edition = "2024"
"#;

const LIBRARY: &str = r#"//! Projected-place fixture.
pub struct Inner { pub value: usize }
pub struct State {
    pub values: [usize; 2],
    pub index: usize,
    pub outer: Inner,
    pub pointer: Box<Inner>,
    pub tuple: (usize,),
}
#[path = "owner.rs"] mod owner;
#[path = "trespasser.rs"] mod trespasser;
"#;

const OWNER: &str = r"//! Declared projected-place owner.
use crate::State;
impl State {
    fn own(&mut self, next: usize) {
        self.values[0] = next;
        self.outer.value = next;
        (*self.pointer).value = next;
        self.tuple.0 = next;
        let _ = self.index;
    }
}
";

const TRESPASSER: &str = r"//! Projected-place trespasser.
use crate::State;
impl State {
    fn trespass(&mut self, next: usize) {
        self.values[self.index] = next;
        self.values[self.index] += 1;
        self.outer.value = next;
        (*self.pointer).value = next;
        self.tuple.0 = next;
        let _ = std::mem::replace(&mut self.values[self.index], next);
    }
}
";

const CONTRACT: &str = r#"schema = 1
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
cycles = "deny"
[source.rust]
module_docs = "required"
facades = "allow"
tests = "allow"
[source.rust.macros]
mode = "allow"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"

[[owner]]
name = "values-authority"
kind = "field-authority"
within = ["src/**"]
match = "crate::State::values"
allow = ["src/owner.rs"]
reason = "Backing value storage stays centralized."

[[owner]]
name = "outer-authority"
kind = "field-authority"
within = ["src/**"]
match = "crate::State::outer"
allow = ["src/owner.rs"]
reason = "Nested storage stays centralized."

[[owner]]
name = "pointer-authority"
kind = "field-authority"
within = ["src/**"]
match = "crate::State::pointer"
allow = ["src/owner.rs"]
reason = "Indirect storage stays centralized."

[[owner]]
name = "tuple-authority"
kind = "field-authority"
within = ["src/**"]
match = "crate::State::tuple"
allow = ["src/owner.rs"]
reason = "Tuple storage stays centralized."

[[owner]]
name = "index-read"
kind = "field-read"
within = ["src/**"]
match = "crate::State::index"
allow = ["src/owner.rs"]
reason = "Index reads stay centralized."
"#;
