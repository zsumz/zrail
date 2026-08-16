//! Contracts shared by effective crate-root integration tests.

pub(super) const WORKSPACE: &str =
    "[workspace]\nmembers = [\"crates/app\", \"crates/tokio\"]\nresolver = \"3\"\n";
pub(super) const PACKAGE: &str =
    "[package]\nname = \"app\"\nversion = \"0.0.0\"\nedition = \"2024\"\n";
pub(super) const UNCONSTRAINED_CONTRACT: &str = r#"schema = 1
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
"#;
pub(super) const BASE_CONTRACT: &str = r#"schema = 1
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

[profiles.pure.effects]
deny = ["process"]

[[layer]]
name = "application"
packages = ["app"]
profiles = ["pure"]
reason = "The application cannot launch processes."
"#;
