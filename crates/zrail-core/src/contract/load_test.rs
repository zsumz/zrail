//! Contract loading is strict, deterministic, and repository-bounded.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{CrateRootSource, DependencyEdgeKind, DependencyReachability};

use super::{load_contract, load_contract_with_entry};

#[test]
fn entry_overlay_preserves_identity_and_imports_without_writing() {
    let root = fixture_root("contract-overlay");
    reset(&root);
    fs::create_dir_all(root.join("zrail.d")).expect("create fragments");
    let original = base_contract("zrail.d/layer.toml");
    fs::write(root.join("zrail.toml"), &original).expect("write root contract");
    fs::write(root.join("zrail.d/layer.toml"), "# preserved fragment\n").expect("write fragment");
    let patched = format!("{original}\n# proposed bytes\n");

    let bundle = load_contract_with_entry(&root, Path::new("zrail.toml"), &patched)
        .expect("load proposed contract");

    assert_eq!(bundle.sources[1].path, "zrail.toml");
    assert_eq!(bundle.sources[1].content, patched);
    assert_eq!(
        fs::read_to_string(root.join("zrail.toml")).unwrap(),
        original
    );
    reset(&root);
}

#[test]
fn local_fragments_merge_without_order_dependent_overrides() {
    let root = fixture_root("contract-merge");
    reset(&root);
    fs::create_dir_all(root.join("zrail.d")).expect("create fragments");
    fs::write(root.join("zrail.toml"), base_contract("zrail.d/*.toml"))
        .expect("write root contract");
    fs::write(
        root.join("zrail.d/layer.toml"),
        r#"[[layer]]
name = "core"
packages = ["fixture"]
may_depend_on = []
reason = "one deterministic layer"
"#,
    )
    .expect("write fragment");

    let bundle = load_contract(&root, Path::new("zrail.toml")).expect("load contract");
    assert_eq!(bundle.contract.layers.len(), 1);
    assert_eq!(bundle.sources.len(), 2);
    reset(&root);
}

#[test]
fn wildcard_imports_do_not_traverse_unrelated_repository_trees() {
    let root = fixture_root("contract-prefix");
    reset(&root);
    fs::create_dir_all(root.join("architecture")).expect("create contract directory");
    fs::write(
        root.join("zrail.toml"),
        base_contract("./architecture/*.toml"),
    )
    .expect("write root contract");
    fs::write(root.join("architecture/empty.toml"), "# fragment\n").expect("write fragment");
    fs::create_dir_all(root.join("node_modules")).expect("create unrelated tree");
    for index in 0..128 {
        fs::write(
            root.join(format!("node_modules/{index}.toml")),
            "unrelated = true\n",
        )
        .expect("write unrelated file");
    }

    let bundle = load_contract(&root, Path::new("zrail.toml")).expect("load narrow imports");

    assert_eq!(bundle.sources.len(), 2);
    reset(&root);
}

#[test]
fn unknown_keys_fail_closed() {
    let root = fixture_root("contract-unknown");
    reset(&root);
    fs::create_dir_all(&root).expect("create root");
    let source = format!("{}\nunknown = true\n", base_contract(""));
    fs::write(root.join("zrail.toml"), source).expect("write contract");

    let error = load_contract(&root, Path::new("zrail.toml")).expect_err("unknown key must fail");
    assert!(error.to_string().contains("unknown field"));
    reset(&root);
}

#[test]
fn profile_reachability_accepts_only_closed_policy_values() {
    let root = fixture_root("profile-reachability");
    reset(&root);
    fs::create_dir_all(&root).expect("create root");
    let source = format!(
        concat!(
            "{}\n",
            "[profiles.restricted]\nreachability = \"tests\"\n",
            "[profiles.restricted.effects]\ndeny = [\"process\"]\n",
            "[[layer]]\nname = \"app\"\npackages = [\"fixture\"]\n",
            "profiles = [\"restricted\"]\nreason = \"Fixture.\"\n",
        ),
        base_contract("")
    );
    fs::write(root.join("zrail.toml"), source).expect("write contract");

    let error =
        load_contract(&root, Path::new("zrail.toml")).expect_err("unknown reachability must fail");

    assert!(error.to_string().contains("unknown variant `tests`"));
    reset(&root);
}

#[test]
fn dependency_rules_default_to_direct_and_accept_transitive_kinds() {
    let root = fixture_root("dependency-reachability");
    reset(&root);
    fs::create_dir_all(&root).expect("create root");
    let source = format!(
        concat!(
            "{}\n",
            "[[dependency]]\nname = 'direct'\nfrom = 'app'\n",
            "deny = ['blocked']\nreason = 'direct boundary'\n\n",
            "[[dependency]]\nname = 'transitive'\nfrom = 'tool'\n",
            "deny = ['blocked']\nreachability = 'transitive'\n",
            "kinds = ['normal', 'build']\nreason = 'resolved boundary'\n",
        ),
        base_contract("")
    );
    fs::write(root.join("zrail.toml"), source).expect("write contract");

    let contract = load_contract(&root, Path::new("zrail.toml"))
        .expect("load dependency policy")
        .contract;

    assert_eq!(
        contract.dependency_rules[0].reachability,
        DependencyReachability::Direct
    );
    assert!(contract.dependency_rules[0].kinds.is_empty());
    assert_eq!(
        contract.dependency_rules[1].reachability,
        DependencyReachability::Transitive
    );
    assert_eq!(
        contract.dependency_rules[1].kinds,
        [DependencyEdgeKind::Normal, DependencyEdgeKind::Build]
    );
    reset(&root);
}

#[test]
fn cargo_lock_source_selector_accepts_exact_discriminators() {
    let root = fixture_root("cargo-lock-source");
    reset(&root);
    fs::create_dir_all(&root).expect("create root");
    let source = format!(
        concat!(
            "{}\n",
            "[source.rust.macros]\nmode = 'deny-unreviewed'\n",
            "[[source.rust.macros.allow]]\nname = 'codegen_macro::generate'\n",
            "resolution = 'exact'\nnamespace_effect = 'none'\n",
            "reason = 'reviewed locked expansion'\n",
            "[source.rust.macros.allow.source]\nkind = 'cargo-lock'\n",
            "package = 'codegen-macro'\nversion = '1.2.3'\n",
            "source = 'registry+https://example.test/index'\n",
        ),
        base_contract("")
    );
    fs::write(root.join("zrail.toml"), source).expect("write contract");

    let contract = load_contract(&root, Path::new("zrail.toml"))
        .expect("load Cargo.lock source selector")
        .contract;

    assert!(matches!(
        contract.source.rust.macros.allow[0]
            .source
            .as_ref()
            .expect("source selector"),
        CrateRootSource::CargoLock { package, version: Some(version), source: Some(source) }
            if package == "codegen-macro"
                && version == "1.2.3"
                && source == "registry+https://example.test/index"
    ));
    reset(&root);
}

#[cfg(unix)]
#[test]
fn contract_symlinks_are_rejected_before_reading() {
    use std::os::unix::fs::symlink;

    let root = fixture_root("contract-symlink");
    reset(&root);
    fs::create_dir_all(&root).expect("create root");
    fs::write(root.join("actual.toml"), base_contract("")).expect("write target");
    symlink(root.join("actual.toml"), root.join("zrail.toml")).expect("create alias");

    let error = load_contract(&root, Path::new("zrail.toml")).expect_err("alias must fail");

    assert!(error.to_string().contains("symlink"));
    reset(&root);
}

#[test]
fn contract_import_depth_is_bounded() {
    let root = fixture_root("contract-depth");
    reset(&root);
    fs::create_dir_all(root.join("zrail.d")).expect("create fragments");
    fs::write(root.join("zrail.toml"), base_contract("zrail.d/0.toml"))
        .expect("write root contract");
    for index in 0..70 {
        let next = index + 1;
        fs::write(
            root.join(format!("zrail.d/{index}.toml")),
            format!("imports = [\"zrail.d/{next}.toml\"]\n"),
        )
        .expect("write fragment");
    }

    let error = load_contract(&root, Path::new("zrail.toml")).expect_err("depth must fail");

    assert!(error.to_string().contains("level safety limit"));
    reset(&root);
}

fn base_contract(import: &str) -> String {
    let imports = if import.is_empty() {
        String::new()
    } else {
        format!("imports = [\"{import}\"]\n")
    };
    format!(
        r#"schema = 1
adapters = ["rust"]
{imports}
[repository]
roots = ["crates"]
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

[source.rust.size.facade]
target = 80
hard = 120
[source.rust.size.implementation]
target = 240
hard = 300
[source.rust.size.test]
target = 300
hard = 400
[source.rust.size.auxiliary]
target = 300
hard = 300
"#
    )
}

fn fixture_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("zrail-{name}-{}", std::process::id()))
}

fn reset(path: &Path) {
    if path.exists() {
        fs::remove_dir_all(path).expect("remove old fixture");
    }
}
