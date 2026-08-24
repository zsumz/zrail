//! Generic repository fixture for governed-surface report tests.

pub(super) const MANIFEST: &str = r#"[package]
name = "audit-app"
version = "0.1.0"
edition = "2024"

[dependencies]
bridge = "1"
"#;

pub(super) const LIBRARY: &str =
    "//! Generic audit fixture.\npub struct Record { pub value: usize }\nmod owner;\n";
pub(super) const OWNER: &str = "//! Exact construction owner.\nuse crate::Record;\npub fn build() -> Record { Record { value: 1 } }\npub fn inspect() { let _ = std::fs::metadata(\"state\"); let _ = std::fs::metadata; let _ = std::env::var(\"MODE\"); }\npub fn write_unknown(value: &mut Unknown) { value.token = 2; }\n";
pub(super) const MIRROR: &str =
    concat!("//! Test mirror.\n", "#[", "test] fn mirrors_build() {}\n");
pub(super) const CHECKSUM: &str =
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

pub(super) const LOCK: &str = r#"version = 4

[[package]]
name = "audit-app"
version = "0.1.0"
dependencies = ["bridge"]

[[package]]
name = "bridge"
version = "1.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
dependencies = ["blocked"]

[[package]]
name = "blocked"
version = "3.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
"#;

pub(super) const AMBIGUOUS_LOCK: &str = r#"version = 4
[[package]]
name = "audit-app"
version = "0.1.0"
dependencies = [
 "bridge 1.2.3 (registry+https://github.com/rust-lang/crates.io-index)",
 "bridge 2.0.0 (registry+https://github.com/rust-lang/crates.io-index)",
]
[[package]]
name = "bridge"
version = "1.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
[[package]]
name = "bridge"
version = "2.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
"#;

pub(super) const CONTRACT: &str = r#"schema = 2
adapters = ["rust"]
[repository]
roots = ["."]
exclude = ["target/**", "scratch/**"]
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
entrypoints = "allow"
tests = "allow"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"

[[source.rust.test_mirrors]]
production = "src/lib.rs"
test = "tests/mirror.rs"
name = "mirrors_build"
receipt = "evidence/mirror.json"
inputs = ["Cargo.lock", "Cargo.toml", "src/owner.rs"]
reason = "The test exercises the governed construction path."

[source.rust.test_mirrors.execution]
command = "cargo test --package audit-app --test mirror mirrors_build --target x86_64-unknown-linux-gnu"
package = "audit-app"
default_features = true
features = []
target = "x86_64-unknown-linux-gnu"
toolchain = "rustc 1.90.0 (example 2026-01-01)"

[[owner]]
name = "record-construction"
kind = "type-construction"
within = ["src/**"]
match = "crate::Record"
allow = ["src/owner.rs"]
reason = "Record construction stays centralized."

[[owner]]
name = "unknown-token-authority"
kind = "field-authority"
within = ["src/**"]
match = "crate::External::token"
allow = ["src/owner.rs"]
reason = "Unresolved operations remain visible to audit consumers."

[[owner]]
name = "metadata-call"
kind = "call"
within = ["src/**"]
match = "std::fs::metadata"
allow = ["src/owner.rs"]
reason = "Metadata access stays visible as calls and references."

[[owner]]
name = "environment-use"
kind = "capability"
within = ["src/**"]
match = "std::env"
allow = ["src/owner.rs"]
reason = "Environment capability use stays centralized."

[[owner]]
name = "artifact-directories"
kind = "directory"
match = "artifacts/*"
allow = ["artifacts/owned"]
reason = "Artifact directories have one reviewed owner."

[[dependency]]
name = "blocked-path"
from = "audit-app"
deny = ["blocked"]
reachability = "transitive"
kinds = ["normal"]
reason = "The resolved path is prohibited."
"#;
