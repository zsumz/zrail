//! Trusted full-snapshot qualification binds original oracles, native observations and execution inputs.

use super::{family::Family, fixtures, model, native, selection};

pub(super) fn run(family: Family) {
    use std::{fs, io::Write as _, path::PathBuf, process::Command};
    use zrail_core::sha256_hex;

    let project = model::project();
    let snapshots =
        PathBuf::from(std::env::var_os("ZRAIL_RC9_SNAPSHOTS").expect("prefetched snapshots"));
    let output = PathBuf::from(std::env::var_os(family.report_env()).expect("fresh report path"));
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
    let (policy, policy_sha256) = model::policy_for(family);
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
    let legacy_counts = family.repository(&source, &roots);
    family.check(legacy_counts.clone());
    assert_eq!(family.native_counts(&observation), legacy_counts);
    let detector = model::detector_source();
    let detector_counts = family.observed("src/reactor/rogue.rs", &detector);
    family.check_detector(detector_counts.clone());
    let fixture_root = evidence.join(format!("transport-{}-parity-fixture", family.name()));
    let fixtures = fixtures::qualify_family(&fixture_root, &roots, &inputs, &policy, family);
    let (total, accepted) = family.totals();
    assert_eq!(fixtures.len(), total);
    assert_eq!(
        fixtures.iter().filter(|row| row.native_accepted).count(),
        accepted
    );
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
        schema: 1, legacy_measure: family.legacy_measure(), implementation_commit, implementation_tree, snapshot, policy_sha256,
        registry_sha256: sha256_hex(&registry_bytes),
        fixture_origins_sha256: sha256_hex(&fs::read(project.join(format!("crates/zrail-testkit/tests/fixtures/rc9/transport-{}-origins.json", family.name()))).expect("origins")),
        test_binary_sha256: sha256_hex(&fs::read(std::env::current_exe().expect("test binary")).expect("binary bytes")),
        cargo_lock_sha256: sha256_hex(&fs::read(project.join("Cargo.lock")).expect("compiler lock")),
        rustc_version: String::from_utf8(compiler.stdout).expect("compiler identity").trim().into(),
        all_source_inputs: model::hashes(&inputs), observation, legacy_counts,
        detector_source_sha256: sha256_hex(detector.as_bytes()), detector_counts,
        fixture_root: fixture_root.to_str().expect("UTF-8 fixture root").into(), fixtures,
        full_repository_qualified: false,
        limitations: vec![
            "Only selected authored syntax quantities and strict Rust file parsing are qualified; no Cargo resolution, semantic receiver identity, execution, lock, or complete repository qualification is claimed.".into(),
            "Original source traversal and collector bodies execute with an injected snapshot root and the original roots loop; the registry and every original Rust source are hash-bound.".into(),
            "Mutations use isolated physical copies; original snapshots and reviewed lock authority remain unchanged.".into(),
            "When legacy_measure is distinct-owner-files, legacy count maps encode one membership per file/subject pair; native observation counts retain actual syntax quantities independently.".into(),
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
        "transport {} parity: {total} fixtures, {accepted} accepted, {} rejected, payload {}",
        family.name(),
        total - accepted,
        sha256_hex(&bytes)
    )
    .expect("write qualification summary");
}
