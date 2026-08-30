//! Repository macro authority binds regular data files and explicit ambient-input review.

use std::{fs, path::Path};

use zrail_rust::{build_lock, check_repository};

#[test]
fn provider_json_and_transitive_helper_template_invalidate_authority() {
    for path in [
        "macros/schema.json",
        "helper/template.txt",
        "macros/templates/excluded.txt",
    ] {
        let root = fixture(&format!("data-{}", path.replace('/', "-")), "");
        let before = build_lock(&root, "zrail.toml".as_ref()).unwrap();
        before.write(&root.join("zrail.lock")).unwrap();
        assert!(
            before
                .macro_implementations
                .iter()
                .all(|record| record.inputs_sha256.len() == 64)
        );
        write(&root, path, "changed implementation input");
        let result = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref()).unwrap();
        assert!(
            result
                .report
                .findings
                .iter()
                .any(|finding| finding.id == "LOCK-023"),
            "{}",
            result.report.human()
        );
        reset(&root);
    }
}

#[test]
fn external_reviewed_inputs_are_bound_and_empty_globs_are_rejected() {
    let root = fixture("extra", "inputs = [\"schemas/**\"]");
    let before = build_lock(&root, "zrail.toml".as_ref()).unwrap();
    before.write(&root.join("zrail.lock")).unwrap();
    write(&root, "schemas/api.json", "changed schema");
    let result = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref()).unwrap();
    assert!(
        result
            .report
            .findings
            .iter()
            .any(|finding| finding.id == "LOCK-023")
    );
    reset(&root);

    let root = fixture("missing", "inputs = [\"missing/**\"]");
    let error = build_lock(&root, "zrail.toml".as_ref())
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("matches no bounded regular files"),
        "{error}"
    );
    reset(&root);
}

#[test]
fn ambient_input_assumption_must_be_explicit_and_closed() {
    let root = fixture("ambient", "");
    let contract = fs::read_to_string(root.join("zrail.toml")).unwrap();
    for replacement in ["", "ambient_inputs = \"unrestricted\""] {
        write(
            &root,
            "zrail.toml",
            &contract.replace("ambient_inputs = \"none\"", replacement),
        );
        let error = build_lock(&root, "zrail.toml".as_ref())
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("ambient_inputs") || error.contains("unrestricted"),
            "{error}"
        );
    }
    write(
        &root,
        "zrail.toml",
        contract
            .split("[source.rust.macros.allow.source]")
            .next()
            .unwrap(),
    );
    let result = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref()).unwrap();
    assert!(
        result
            .report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-MACRO-006"),
        "{}",
        result.report.human()
    );
    reset(&root);
}

#[test]
fn input_patterns_cannot_escape_select_reserved_outputs_or_repeat() {
    for inputs in [
        "inputs = [\"../secret\"]",
        "inputs = [\"target/**\"]",
        "inputs = [\"zrail.lock\"]",
        "inputs = [\"schemas/**\", \"schemas/**\"]",
    ] {
        let root = fixture(&format!("invalid-{}", inputs.len()), inputs);
        assert!(build_lock(&root, "zrail.toml".as_ref()).is_err());
        reset(&root);
    }
}

#[cfg(unix)]
#[test]
fn owned_and_explicit_symlinks_are_not_silently_omitted() {
    for path in ["macros/input-link", "schemas/input-link"] {
        let root = fixture(
            &format!("link-{}", path.replace('/', "-")),
            "inputs = [\"schemas/**\"]",
        );
        std::os::unix::fs::symlink(root.join("schemas/api.json"), root.join(path)).unwrap();
        let error = build_lock(&root, "zrail.toml".as_ref())
            .unwrap_err()
            .to_string();
        assert!(error.contains("not a regular file"), "{error}");
        reset(&root);
    }
}

fn fixture(name: &str, extra: &str) -> std::path::PathBuf {
    let root =
        std::env::temp_dir().join(format!("zrail-macro-inputs-{name}-{}", std::process::id()));
    reset(&root);
    for directory in [
        "app/src",
        "macros/src",
        "macros/templates",
        "helper/src",
        "schemas",
    ] {
        fs::create_dir_all(root.join(directory)).unwrap();
    }
    write(
        &root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"app\", \"macros\", \"helper\"]\nresolver = \"3\"\n",
    );
    write(
        &root,
        "app/Cargo.toml",
        "[package]\nname = \"app\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\nreviewed = { path = \"../macros\" }\n",
    );
    write(
        &root,
        "macros/Cargo.toml",
        "[package]\nname = \"reviewed\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[lib]\nproc-macro = true\n[dependencies]\nhelper = { path = \"../helper\" }\n",
    );
    write(
        &root,
        "helper/Cargo.toml",
        "[package]\nname = \"helper\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    );
    write(
        &root,
        "app/src/lib.rs",
        "pub fn generated() { reviewed::generate!(); }\n",
    );
    write(
        &root,
        "macros/src/lib.rs",
        r#"#[proc_macro]
pub fn generate(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(root.join("schema.json")).unwrap().parse().unwrap()
}
"#,
    );
    write(&root, "helper/src/lib.rs", "pub fn helper() {}\n");
    for path in [
        "macros/schema.json",
        "helper/template.txt",
        "macros/templates/excluded.txt",
        "schemas/api.json",
    ] {
        write(&root, path, "");
    }
    write(
        &root,
        "zrail.toml",
        &CONTRACT.replace("EXTRA_INPUTS", extra),
    );
    root
}

fn write(root: &Path, path: &str, content: &str) {
    fs::write(root.join(path), content).unwrap();
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
}

const CONTRACT: &str = r#"schema = 2
adapters = ["rust"]
[repository]
roots = ["."]
exclude = ["macros/templates/**"]
workspace_members = "exact"
nested_git = "deny"
submodules = "deny"
symlinks = "inside"
[dependencies]
mode = "observed"
unassigned_packages = "allow"
cycles = "deny"
[source.rust]
module_docs = "allow"
facades = "allow"
tests = "allow"
[source.rust.hygiene]
unsafe = "allow"
lint_suppressions = "allow"
[source.rust.macros]
mode = "deny-unreviewed"
[[source.rust.macros.allow]]
name = "env"
reason = "Cargo's manifest directory only locates the bound package input tree."
[[source.rust.macros.allow]]
name = "reviewed::generate"
source_operations = "none"
reason = "Reviewed output and input boundary, including owned JSON and helper templates."
[source.rust.macros.allow.source]
kind = "repository"
package = "reviewed"
directory = "macros"
ambient_inputs = "none"
EXTRA_INPUTS
"#;
