//! Host-side proc-macro implementation effects are not application runtime effects.

use std::{fs, path::PathBuf};

use zrail_rust::check_repository;

#[test]
fn runtime_profiles_exclude_proc_macros_but_all_profiles_govern_them() {
    for target in ["proc-macro = true", "crate-type = [\"proc-macro\"]"] {
        for reachability in ["production", "all"] {
            let root = fixture(target, reachability);
            let result =
                check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref()).unwrap();
            let effects = result
                .report
                .findings
                .iter()
                .filter(|finding| finding.id == "EFFECT-001")
                .collect::<Vec<_>>();
            if reachability == "production" {
                assert!(effects.is_empty(), "{}", result.report.human());
            } else {
                for effect in ["Filesystem", "Process"] {
                    assert!(
                        effects
                            .iter()
                            .any(|finding| finding.message.contains(effect)),
                        "{}",
                        result.report.human()
                    );
                }
            }
            fs::remove_dir_all(root).unwrap();
        }
    }
}

fn fixture(target: &str, reachability: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-proc-runtime-{}-{reachability}-{}",
        target.len(),
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("Cargo.toml"), format!(
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[lib]\n{target}\n"
    )).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        r#"//! Host implementation.
#[proc_macro]
pub fn generate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = std::fs::read_to_string("schema.json");
    let _ = std::process::Command::new("helper");
    input
}
"#,
    )
    .unwrap();
    fs::write(
        root.join("zrail.toml"),
        CONTRACT.replace("REACHABILITY", reachability),
    )
    .unwrap();
    root
}

const CONTRACT: &str = r#"schema = 2
adapters = ["rust"]
[repository]
roots = ["."]
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
[profiles.runtime]
reachability = "REACHABILITY"
[profiles.runtime.effects]
deny = ["filesystem", "process"]
[[layer]]
name = "implementation"
packages = ["fixture"]
profiles = ["runtime"]
reason = "Govern compile-time implementation only when all reachability is selected."
"#;
