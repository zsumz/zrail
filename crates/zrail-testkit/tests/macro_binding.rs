//! Macro authority binds one invocation across exact, globbed, and unresolved candidates.

use std::{fs, path::Path};

use zrail_core::ReportStatus;
use zrail_rust::{build_lock, check_repository};

#[test]
fn sibling_repository_glob_resolves_to_one_exact_allowance() {
    let root = repository(
        "repository-glob",
        r"//! Repository glob fixture.
mod support {
    macro_rules! reviewed { () => { 1 }; }
    pub(crate) use reviewed;
}
#[cfg(test)]
mod tests {
    use super::support::*;
    #[test]
    fn expands() { let _ = reviewed!(); }
}
",
        r#"
[[source.rust.macros.allow]]
name = "super::support::reviewed"
definition = "src/lib.rs"
reason = "Reviewed repository assertion expansion."
"#,
    );
    lock(&root);

    let report = check(&root);

    assert_eq!(report.status, ReportStatus::Pass, "{:#?}", report.findings);
    reset(&root);
}

#[test]
fn sibling_glob_resolves_parent_exact_import() {
    let root = repository(
        "sibling-exact-import",
        "//! Exact parent import fixture.\nuse reviewed_json::json as reviewed;\n#[cfg(test)]\nmod reviewed_test;\n",
        r#"
[[source.rust.macros.allow]]
name = "serde_json::json"
inputs = "opaque"
reason = "Reviewed dependency expansion boundary."
[source.rust.macros.allow.source]
kind = "registry"
requirement = "1"
"#,
    );
    fs::write(
        root.join("Cargo.toml"),
        format!(
            "{MANIFEST}\n[dependencies]\nreviewed_json = {{ package = \"serde_json\", version = \"1\" }}\n"
        ),
    )
    .expect("write dependency manifest");
    fs::write(
        root.join("src/reviewed_test.rs"),
        "//! Sibling glob fixture.\nuse super::*;\n#[test]\nfn expands() { let _ = reviewed!({\"ok\": true}); }\n",
    )
    .expect("write sibling test module");
    lock(&root);

    let report = check(&root);

    assert_eq!(report.status, ReportStatus::Pass, "{:#?}", report.findings);
    reset(&root);
}

#[test]
fn test_only_dependency_reexport_cannot_authorize_a_production_macro() {
    let root = repository(
        "guarded-reexport",
        "//! Guarded re-export fixture.\n#[cfg(test)] pub use reviewed_json::json as reviewed;\npub fn run() { let _ = reviewed!({\"ok\": true}); }\n",
        r#"
[[source.rust.macros.allow]]
name = "serde_json::json"
inputs = "opaque"
reason = "Reviewed dependency expansion boundary."
[source.rust.macros.allow.source]
kind = "registry"
requirement = "1"
"#,
    );
    fs::write(
        root.join("Cargo.toml"),
        format!(
            "{MANIFEST}\n[dependencies]\nreviewed_json = {{ package = \"serde_json\", version = \"1\" }}\n"
        ),
    )
    .expect("write dependency manifest");
    lock(&root);

    let report = check(&root);

    assert!(
        report.findings.iter().any(|finding| {
            matches!(finding.id.as_str(), "RUST-MACRO-001" | "RUST-MACRO-006")
                && finding.message.contains("reviewed")
        }),
        "{:#?}",
        report.findings
    );
    reset(&root);
}

#[test]
fn ambiguous_globs_emit_one_diagnostic_until_every_candidate_is_allowed() {
    let source = r"//! Ambiguous glob fixture.
mod one { macro_rules! reviewed { () => { 1 }; } pub(crate) use reviewed; }
mod two { macro_rules! reviewed { () => { 2 }; } pub(crate) use reviewed; }
#[cfg(test)]
mod tests {
    use super::{one::*, two::*};
    #[test]
    fn expands() { reviewed! { token dsl } }
}
";
    let one = opaque_allowance("super::one::reviewed");
    let root = repository("ambiguous-globs", source, &one);
    lock(&root);

    let denied = check(&root);
    assert_eq!(
        denied
            .findings
            .iter()
            .filter(|finding| {
                matches!(
                    finding.id.as_str(),
                    "RUST-MACRO-001" | "RUST-MACRO-003" | "RUST-MACRO-006"
                )
            })
            .count(),
        1,
        "{:#?}",
        denied.findings
    );

    fs::write(
        root.join("zrail.toml"),
        format!(
            "{CONTRACT}{}{}",
            opaque_allowance("super::one::reviewed"),
            opaque_allowance("super::two::reviewed")
        ),
    )
    .expect("write complete glob authority");
    lock(&root);
    let allowed = check(&root);
    assert_eq!(
        allowed.status,
        ReportStatus::Pass,
        "{:#?}",
        allowed.findings
    );
    reset(&root);
}

#[test]
fn unresolved_written_name_requires_explicit_conservative_binding() {
    let source = "//! Unresolved macro fixture.\npub fn run() { unknown!(); }\n";
    let exact = allowance("unknown");
    let root = repository("unresolved-exact", source, &exact);
    lock(&root);

    let denied = check(&root);
    assert_eq!(
        denied
            .findings
            .iter()
            .filter(|finding| finding.id == "RUST-MACRO-006")
            .count(),
        1,
        "{:#?}",
        denied.findings
    );

    fs::write(
        root.join("zrail.toml"),
        format!(
            "{CONTRACT}{}",
            allowance_with_binding("unknown", "conservative")
        ),
    )
    .expect("write conservative authority");
    lock(&root);
    let allowed = check(&root);
    assert_eq!(
        allowed.status,
        ReportStatus::Pass,
        "{:#?}",
        allowed.findings
    );
    reset(&root);
}

fn allowance(name: &str) -> String {
    format!(
        "\n[[source.rust.macros.allow]]\nname = \"{name}\"\nreason = \"Reviewed expansion boundary.\"\n"
    )
}

fn allowance_with_binding(name: &str, binding: &str) -> String {
    format!(
        "\n[[source.rust.macros.allow]]\nname = \"{name}\"\nbinding = \"{binding}\"\nreason = \"Reviewed unresolved expansion boundary.\"\n"
    )
}

fn opaque_allowance(name: &str) -> String {
    format!(
        "\n[[source.rust.macros.allow]]\nname = \"{name}\"\ninputs = \"opaque\"\nreason = \"Reviewed opaque expansion boundary.\"\n[source.rust.macros.allow.source]\nkind = \"repository\"\npackage = \"fixture\"\ndirectory = \".\"\nambient_inputs = \"none\"\n"
    )
}

fn repository(name: &str, source: &str, allowances: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-binding-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source root");
    fs::write(root.join("Cargo.toml"), MANIFEST).expect("write manifest");
    fs::write(root.join("src/lib.rs"), source).expect("write source");
    fs::write(root.join("zrail.toml"), format!("{CONTRACT}{allowances}")).expect("write contract");
    root
}

fn lock(root: &Path) {
    build_lock(root, "zrail.toml".as_ref())
        .expect("build fixture lock")
        .write(&root.join("zrail.lock"))
        .expect("write fixture lock");
}

fn check(root: &Path) -> zrail_core::Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check fixture")
        .report
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const MANIFEST: &str = "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n";

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
mode = "deny-unreviewed"

[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
deny_methods = []
deny_macros = []
"#;
