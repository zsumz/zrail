//! Frozen collector parity checks syntax corners before downstream replacement is claimed.

#[path = "rust_inventories/parity/expression_cases.rs"]
mod expression_cases;
#[path = "rust_inventories/parity/legacy.rs"]
mod legacy;
#[path = "rust_inventories/parity/native.rs"]
mod native;
#[path = "rust_inventories/parity/selection.rs"]
mod selection;
#[path = "rust_inventories/parity/syntax_cases.rs"]
mod syntax_cases;

#[path = "rust_inventories/parity/fixtures.rs"]
mod fixtures;
#[path = "rust_inventories/parity/model.rs"]
mod model;
#[path = "rust_inventories/parity/mutations.rs"]
mod mutations;

#[test]
fn frozen_transport_methods_match_every_parsed_expression_context() {
    assert_eq!(legacy::expected().len(), 11);
    assert_eq!(legacy::expected().values().sum::<usize>(), 20);
    let root = std::env::temp_dir().join(format!(
        "zrail-method-parity-{}-{:?}",
        std::process::id(),
        std::thread::current().id(),
    ));
    for (index, source) in syntax_cases::CASES.into_iter().enumerate() {
        let expected = legacy::observed("src/sample.rs", source);
        let observed = native::observed(&root, source);
        assert_eq!(observed, expected, "case {index}: {source}");
    }
}

#[test]
fn frozen_transport_assertion_rejects_each_quantity_and_scope_regression() {
    use std::fmt::Write as _;

    let (policy, _) = model::policy();
    let roots = policy
        .include
        .iter()
        .map(|pattern| {
            pattern
                .strip_suffix("/**/*.rs")
                .expect("exact frozen root selector")
                .into()
        })
        .collect::<Vec<String>>();
    let mut sources = std::collections::BTreeMap::<String, String>::new();
    for (key, count) in legacy::expected() {
        let (path, method) = key.rsplit_once(':').expect("reviewed pair");
        let source = sources.entry(path.into()).or_default();
        for index in 0..count {
            writeln!(source, "fn site_{method}_{index}(x: X) {{ x.{method}(); }}")
                .expect("write synthetic source");
        }
    }
    let root = std::env::temp_dir().join(format!(
        "zrail-transport-fixtures-{}-{:?}",
        std::process::id(),
        std::thread::current().id(),
    ));
    let cases = fixtures::qualify(&root, &roots, &sources, &policy);
    assert_eq!(cases.len(), 76);
    assert_eq!(cases.iter().filter(|row| row.native_accepted).count(), 7);
    let detector = legacy::observed("src/reactor/rogue.rs", &model::detector_source());
    legacy::check_detector_methods(detector);
}

#[test]
#[ignore = "requires prefetched snapshots and fresh ZRAIL_RC9_TRANSPORT_METHODS_REPORT"]
fn qualify_all_frozen_kafka_driver_transport_methods() {
    use std::{fs, io::Write as _, path::PathBuf, process::Command};
    use zrail_core::sha256_hex;

    let project = model::project();
    let snapshots =
        PathBuf::from(std::env::var_os("ZRAIL_RC9_SNAPSHOTS").expect("prefetched snapshots"));
    let output = PathBuf::from(
        std::env::var_os("ZRAIL_RC9_TRANSPORT_METHODS_REPORT").expect("fresh report path"),
    );
    assert!(!output.exists(), "never overwrite evidence");
    let evidence =
        fs::canonicalize(output.parent().expect("report parent")).expect("evidence directory");
    assert!(!evidence.starts_with(fs::canonicalize(&project).expect("project")));
    assert!(!evidence.starts_with(fs::canonicalize(&snapshots).expect("snapshots")));
    assert!(
        Command::new("python3")
            .arg(project.join("scripts/rc9-snapshots"))
            .arg(&snapshots)
            .status()
            .expect("offline verification")
            .success()
    );
    let git = |args: &[&str]| {
        let result = Command::new("git")
            .arg("-C")
            .arg(&project)
            .args(args)
            .output()
            .expect("Git identity");
        assert!(result.status.success());
        String::from_utf8(result.stdout)
            .expect("UTF-8 Git identity")
            .trim()
            .to_owned()
    };
    assert!(
        git(&["status", "--porcelain=v1", "--untracked-files=all"]).is_empty(),
        "qualify a committed tree"
    );
    let implementation_commit = git(&["rev-parse", "HEAD"]);
    let implementation_tree = git(&["rev-parse", "HEAD^{tree}"]);
    let source = snapshots.join("kafka-driver");
    let registry_bytes = fs::read(source.join("guardrails.toml")).expect("frozen registry");
    let registry: toml::Value =
        toml::from_str(std::str::from_utf8(&registry_bytes).expect("UTF-8 registry"))
            .expect("registry");
    let roots: Vec<String> = registry["paths"]["rust_roots"]
        .clone()
        .try_into()
        .expect("source roots");
    let inputs = model::inputs(&source, &roots);
    assert_eq!(inputs.len(), 772);
    let (policy, policy_sha256) = model::policy();
    let observation = native::analyze(&source, &inputs, &policy).expect("complete selected syntax");
    assert!(observation.satisfied);
    let mut selected = crate::rules::legacy_driver_paths(&source, &roots)
        .into_iter()
        .filter(|path| !selection::is_test(&source, path))
        .map(|path| selection::display_path(&source, &path))
        .collect::<Vec<_>>();
    selected.sort();
    assert_eq!(
        selected,
        observation
            .inputs
            .iter()
            .map(|input| input.path.clone())
            .collect::<Vec<_>>()
    );
    let legacy_counts = legacy::repository(&source, &roots);
    legacy::check_selector_methods(legacy_counts.clone());
    assert_eq!(
        observation
            .counts
            .iter()
            .map(|count| (format!("{}:{}", count.path, count.name), count.count))
            .collect::<std::collections::BTreeMap<_, _>>(),
        legacy_counts
    );
    let detector = model::detector_source();
    let detector_counts = legacy::observed("src/reactor/rogue.rs", &detector);
    legacy::check_detector_methods(detector_counts.clone());
    let fixture_root = evidence.join("transport-methods-parity-fixture");
    let fixtures = fixtures::qualify(&fixture_root, &roots, &inputs, &policy);
    assert_eq!(fixtures.len(), 76);
    assert_eq!(fixtures.iter().filter(|row| row.native_accepted).count(), 7);
    let pins: serde_json::Value = serde_json::from_slice(
        &fs::read(project.join("crates/zrail-testkit/tests/fixtures/rc9/snapshots.json"))
            .expect("pins"),
    )
    .expect("pin schema");
    let snapshot = pins["consumers"]
        .as_array()
        .expect("consumer pins")
        .iter()
        .find(|pin| pin["name"] == "kafka-driver")
        .expect("driver pin")
        .clone();
    let compiler = Command::new("rustc")
        .arg("--version")
        .current_dir(&project)
        .output()
        .expect("compiler");
    assert!(compiler.status.success());
    let report = model::Report {
        schema: 1, implementation_commit, implementation_tree, snapshot, policy_sha256,
        registry_sha256: sha256_hex(&registry_bytes),
        fixture_origins_sha256: sha256_hex(&fs::read(project.join("crates/zrail-testkit/tests/fixtures/rc9/transport-methods-origins.json")).expect("origins")),
        test_binary_sha256: sha256_hex(&fs::read(std::env::current_exe().expect("test binary")).expect("binary bytes")),
        cargo_lock_sha256: sha256_hex(&fs::read(project.join("Cargo.lock")).expect("compiler lock")),
        rustc_version: String::from_utf8(compiler.stdout).expect("compiler identity").trim().into(),
        all_source_inputs: model::hashes(&inputs), observation, legacy_counts,
        detector_source_sha256: sha256_hex(detector.as_bytes()), detector_counts,
        fixture_root: fixture_root.to_str().expect("UTF-8 fixture root").into(), fixtures,
        full_repository_qualified: false,
        limitations: vec![
            "Only selected authored method syntax and strict Rust file parsing are qualified; no Cargo resolution, semantic receiver identity, execution, lock, or complete repository qualification is claimed.".into(),
            "Original source traversal and collector bodies execute with an injected snapshot root and the original roots loop; the registry and every original Rust source are hash-bound.".into(),
            "Mutations use isolated physical copies; original snapshots and reviewed lock authority remain unchanged.".into(),
        ],
    };
    let mut bytes = serde_json::to_vec_pretty(&report).expect("typed evidence");
    bytes.push(b'\n');
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .expect("fresh output")
        .write_all(&bytes)
        .expect("write evidence");
    assert!(git(&["status", "--porcelain=v1", "--untracked-files=all"]).is_empty());
    assert!(
        Command::new("python3")
            .arg(project.join("scripts/rc9-snapshots"))
            .arg(&snapshots)
            .status()
            .expect("final snapshot verification")
            .success()
    );
    writeln!(
        std::io::stderr(),
        "transport method parity: 76 fixtures, 7 accepted, 69 rejected, payload {}",
        sha256_hex(&bytes)
    )
    .expect("write qualification summary");
}

#[test]
fn frozen_transport_expression_paths_preserve_authored_contexts_and_quantities() {
    let root = std::env::temp_dir().join(format!(
        "zrail-expression-parity-{}-{:?}",
        std::process::id(),
        std::thread::current().id(),
    ));
    for (source, count) in expression_cases::CASES {
        let expected = legacy::associated("src/sample.rs", source);
        assert_eq!(
            expected.values().sum::<usize>(),
            count,
            "frozen semantics: {source}"
        );
        assert_eq!(
            native::observed_with_subject(&root, source, expression_cases::SUBJECT),
            expected,
            "{source}"
        );
    }
}
