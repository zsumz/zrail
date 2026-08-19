Checks a repository against its contract and optional lock without writing files.

`config` and `lock_path` may be relative to `root`; resolved paths must remain
within the repository. A missing required lock becomes a report finding rather
than an I/O error. The returned candidate lock is kept in memory.

# Example

This example creates a minimal repository, checks it in observed mode, and
verifies that checking did not create a lock file.

```rust
use std::{fs, path::Path};
use zrail_rust::check_repository;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let root = std::env::temp_dir().join(format!(
    "zrail-rust-doctest-{}",
    std::process::id()
));
if root.exists() {
    fs::remove_dir_all(&root)?;
}
fs::create_dir_all(root.join("src"))?;
fs::write(
    root.join("Cargo.toml"),
    "[package]\nname = \"demo\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
)?;
fs::write(root.join("src/lib.rs"), "//! Demo crate.\n")?;
fs::write(
    root.join("zrail.toml"),
    r#"schema = 1
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

[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
deny_methods = []
deny_macros = []
"#,
)?;

let checked = check_repository(
    &root,
    Path::new("zrail.toml"),
    Path::new("zrail.lock"),
)?;
assert_eq!(
    checked.report.summary.errors,
    0,
    "{}",
    checked.report.human()
);
assert_eq!(checked.packages, 1);
assert!(!root.join("zrail.lock").exists());
fs::remove_dir_all(root)?;
# Ok(())
# }
```
