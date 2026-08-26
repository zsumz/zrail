//! Field-place resolution keeps feature-world source variants isolated.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use super::check_repository;

static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

#[test]
fn disjoint_feature_worlds_resolve_nested_writes_and_receiver_calls_exactly() {
    let root = repository(DISJOINT_WORLDS);

    let report = report(&root);

    assert_eq!(error_count(&report, "OWN-006"), 0, "{report}");
    assert_eq!(error_count(&report, "OWN-003"), 0, "{report}");
    reset(&root);
}

#[test]
fn same_world_source_variants_remain_ambiguous_and_fail_closed() {
    let root = repository(OVERLAPPING_WORLD);

    let report = report(&root);

    assert!(error_count(&report, "OWN-006") >= 2, "{report}");
    reset(&root);
}

fn repository(worlds: &str) -> PathBuf {
    let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let root =
        std::env::temp_dir().join(format!("zrail-place-domains-{}-{id}", std::process::id()));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture source");
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "zrail.toml", &format!("{CONTRACT}\n{worlds}"));
    write(&root, "src/lib.rs", LIBRARY);
    write(&root, "src/a.rs", &variant("AInner"));
    write(&root, "src/b.rs", &variant("BInner"));
    root
}

fn variant(inner: &str) -> String {
    format!(
        "//! Feature-selected implementation.\n\
         struct {inner} {{ leaf: u64 }}\n\
         struct Outer {{ inner: {inner}, values: Vec<u64> }}\n\
         struct State {{ outer: Outer }}\n\
         impl State {{\n\
             fn mutate(&mut self) {{\n\
                 self.outer.inner.leaf = 1;\n\
                 self.outer.values.push(1);\n\
             }}\n\
         }}\n"
    )
}

fn report(root: &Path) -> String {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check repository")
        .report
        .human()
}

fn error_count(report: &str, id: &str) -> usize {
    report
        .lines()
        .filter(|line| line.starts_with(&format!("error[{id}]")))
        .count()
}

fn write(root: &Path, path: &str, content: &str) {
    fs::write(root.join(path), content).expect("write fixture");
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const MANIFEST: &str = r#"[package]
name = "place-app"
version = "0.0.0"
edition = "2024"

[features]
a = []
b = []
"#;

const LIBRARY: &str = concat!(
    "//! Feature-selected facade.\n",
    "#[cfg(feature = \"a\")]\n",
    "#[path = \"a.rs\"]\nmod ",
    "selected;\n",
    "#[cfg(feature = \"b\")]\n",
    "#[path = \"b.rs\"]\nmod ",
    "selected;\n",
);

const CONTRACT: &str = r#"schema = 2
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

[[owner]]
name = "a-inner-leaf"
kind = "field-mutation"
within = ["src/a.rs"]
match = "crate::selected::AInner::leaf"
mutating_methods = ["push"]
allow = ["src/a.rs"]
reason = "The A feature owns its nested leaf."

[[owner]]
name = "b-inner-leaf"
kind = "field-mutation"
within = ["src/b.rs"]
match = "crate::selected::BInner::leaf"
mutating_methods = ["push"]
allow = ["src/b.rs"]
reason = "The B feature owns its nested leaf."

[[owner]]
name = "a-values"
kind = "field-mutation"
within = ["src/a.rs"]
match = "crate::selected::Outer::values"
mutating_methods = ["push"]
allow = ["src/a.rs"]
reason = "The A feature owns its values."

[[owner]]
name = "b-values"
kind = "field-mutation"
within = ["src/b.rs"]
match = "crate::selected::Outer::values"
mutating_methods = ["push"]
allow = ["src/b.rs"]
reason = "The B feature owns its values."
"#;

const DISJOINT_WORLDS: &str = r#"[[source.rust.feature_worlds]]
name = "a-only"
reason = "The A source variant is supported independently."

[[source.rust.feature_worlds.packages]]
package = "place-app"
default_features = false
features = ["a"]

[[source.rust.feature_worlds]]
name = "b-only"
reason = "The B source variant is supported independently."

[[source.rust.feature_worlds.packages]]
package = "place-app"
default_features = false
features = ["b"]
"#;

const OVERLAPPING_WORLD: &str = r#"[[source.rust.feature_worlds]]
name = "both"
reason = "This adversarial world activates overlapping source variants."

[[source.rust.feature_worlds.packages]]
package = "place-app"
default_features = false
features = ["a", "b"]
"#;
