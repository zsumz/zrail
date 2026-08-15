//! Cargo crate-root discovery covers explicit and conventional targets.

use std::{fs, path::PathBuf};

use toml::Value;

use super::{CargoTargetKind, collect_target_roots};

#[test]
fn conventional_and_explicit_targets_become_crate_roots() {
    let root = fixture_root("conventional");
    reset(&root);
    for path in [
        "src/lib.rs",
        "src/main.rs",
        "src/bin/tool.rs",
        "tests/integration.rs",
        "examples/demo/main.rs",
        "benches/speed.rs",
        "build/custom.rs",
    ] {
        let destination = root.join(path);
        fs::create_dir_all(destination.parent().expect("target parent")).expect("create parent");
        fs::write(destination, "//! target\n").expect("write target");
    }
    let manifest = r#"
        [package]
        name = "fixture"
        version = "0.0.0"
        edition = "2021"
        build = "build/custom.rs"

        [[example]]
        name = "explicit"
        path = "examples/explicit.rs"
    "#
    .parse::<Value>()
    .expect("parse manifest");

    let roots = collect_target_roots(&manifest, &root, None).expect("collect roots");

    for expected in [
        "src/lib.rs",
        "src/main.rs",
        "src/bin/tool.rs",
        "tests/integration.rs",
        "examples/demo/main.rs",
        "examples/explicit.rs",
        "benches/speed.rs",
        "build/custom.rs",
    ] {
        assert!(
            roots.iter().any(|root| root.path == expected),
            "missing {expected}"
        );
    }
    reset(&root);
}

#[test]
fn disabled_auto_discovery_does_not_claim_source_files() {
    let root = fixture_root("disabled");
    reset(&root);
    fs::create_dir_all(root.join("src/bin")).expect("create source");
    fs::write(root.join("src/lib.rs"), "//! library\n").expect("write library");
    fs::write(root.join("src/bin/tool.rs"), "//! binary\n").expect("write binary");
    let manifest = r#"
        [package]
        name = "fixture"
        version = "0.0.0"
        autolib = false
        autobins = false
    "#
    .parse::<Value>()
    .expect("parse manifest");

    assert!(
        collect_target_roots(&manifest, &root, None)
            .expect("collect roots")
            .is_empty()
    );
    reset(&root);
}

#[test]
fn explicit_target_names_override_auto_discovery() {
    let root = fixture_root("explicit-override");
    reset(&root);
    for path in ["src/bin/tool.rs", "commands/tool.rs"] {
        let destination = root.join(path);
        fs::create_dir_all(destination.parent().expect("target parent")).expect("create parent");
        fs::write(destination, "//! target\n").expect("write target");
    }
    let manifest = r#"
        [package]
        name = "fixture"
        version = "0.0.0"
        edition = "2021"

        [[bin]]
        name = "tool"
        path = "commands/tool.rs"
    "#
    .parse::<Value>()
    .expect("parse manifest");

    let roots = collect_target_roots(&manifest, &root, None).expect("collect roots");

    assert!(roots.iter().any(|target| target.path == "commands/tool.rs"));
    assert!(!roots.iter().any(|target| target.path == "src/bin/tool.rs"));
    reset(&root);
}

#[test]
fn ambiguous_auto_discovered_target_names_are_rejected() {
    let root = fixture_root("ambiguous-auto");
    reset(&root);
    fs::create_dir_all(root.join("examples/demo")).expect("create nested target");
    fs::write(root.join("examples/demo.rs"), "//! direct\n").expect("write direct");
    fs::write(root.join("examples/demo/main.rs"), "//! nested\n").expect("write nested");
    fs::write(root.join("examples/upper.RS"), "//! wrong extension\n")
        .expect("write uppercase source");
    let manifest = r#"
        [package]
        name = "fixture"
        version = "0.0.0"
    "#
    .parse::<Value>()
    .expect("parse manifest");

    let error =
        collect_target_roots(&manifest, &root, None).expect_err("ambiguous target must fail");

    assert!(error.contains("ambiguous"));
    reset(&root);
}

#[test]
fn edition_2015_manual_targets_disable_implicit_discovery() {
    let root = fixture_root("edition-2015");
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source");
    fs::write(root.join("src/lib.rs"), "//! implicit library\n").expect("write library");
    fs::write(root.join("custom.rs"), "fn main() {}\n").expect("write explicit target");
    let manifest = r#"
        [package]
        name = "fixture"
        version = "0.0.0"

        [[bin]]
        name = "custom"
        path = "custom.rs"
    "#
    .parse::<Value>()
    .expect("parse manifest");

    let roots = collect_target_roots(&manifest, &root, None).expect("collect roots");

    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0].path, "custom.rs");
    assert_eq!(roots[0].kind, CargoTargetKind::Binary);
    reset(&root);
}

#[test]
fn unknown_editions_fail_closed() {
    let root = fixture_root("unknown-edition");
    reset(&root);
    let manifest = r#"
        [package]
        name = "fixture"
        version = "0.0.0"
        edition = "future"
    "#
    .parse::<Value>()
    .expect("parse manifest");

    assert!(collect_target_roots(&manifest, &root, None).is_err());
}

fn fixture_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("zrail-targets-{}-{name}", std::process::id()))
}

fn reset(root: &std::path::Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}
