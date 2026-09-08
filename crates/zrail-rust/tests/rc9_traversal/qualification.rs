//! A clean producer binds the exact frozen configuration, complete selected paths and failure matrix.

#[cfg(unix)]
use super::model::{self, POLICY, ROOTS};
#[cfg(unix)]
use serde::Serialize;
#[cfg(unix)]
use std::{
    collections::BTreeMap,
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(unix)]
#[derive(Serialize)]
struct Report {
    schema: u32,
    implementation_commit: String,
    implementation_tree: String,
    test_binary_sha256: String,
    rustc_version: String,
    cargo_lock_sha256: String,
    policy_sha256: String,
    snapshot: serde_json::Value,
    full_repository_qualified: bool,
    guardrails_sha256: String,
    support_sha256: String,
    roots: Vec<String>,
    frozen_inputs: BTreeMap<String, String>,
    frozen_paths: Vec<String>,
    frozen_observations: Vec<crate::GovernedRepositoryFile>,
    physical: Vec<model::Row>,
    injected: Vec<super::injected::Row>,
    limitations: [&'static str; 4],
}

#[cfg(unix)]
pub(super) fn run() {
    let project = model::project();
    let snapshots = PathBuf::from(std::env::var_os("ZRAIL_RC9_SNAPSHOTS").expect("snapshots"))
        .canonicalize()
        .expect("snapshots");
    let output =
        PathBuf::from(std::env::var_os("ZRAIL_RC9_TRAVERSAL_REPORT").expect("fresh report"));
    let directory = output
        .parent()
        .expect("parent")
        .canonicalize()
        .expect("external directory");
    assert!(
        !output.exists() && !directory.starts_with(&project) && !directory.starts_with(&snapshots)
    );
    let verify = || {
        assert!(
            Command::new("python3")
                .arg(project.join("scripts/rc9-snapshots"))
                .arg(&snapshots)
                .status()
                .expect("verify snapshots")
                .success()
        );
        assert!(
            git(
                &project,
                &["status", "--porcelain=v1", "--untracked-files=all"]
            )
            .is_empty()
        );
    };
    verify();
    model::source_binding();
    let commit = git(&project, &["rev-parse", "HEAD"]);
    let root = snapshots.join("kafka-driver");
    let hash = |path: &Path| zrail_core::sha256_hex(&fs::read(path).expect("bound input"));
    let guardrails_sha256 = hash(&root.join("guardrails.toml"));
    let support_sha256 = hash(&root.join("tests/guardrails/support.rs"));
    assert_eq!(
        guardrails_sha256,
        "099d48347a22288d024a05260c5a102a4b142a0f1ce7a4d763e8979a83e09f10"
    );
    assert_eq!(
        support_sha256,
        "bb0f3392b649b75ad417923e98559137f28b67e98815cbfa0470bb64808bc843"
    );
    let config: toml::Value =
        toml::from_str(&fs::read_to_string(root.join("guardrails.toml")).expect("config"))
            .expect("frozen TOML");
    let roots = config["paths"]["rust_roots"]
        .as_array()
        .expect("roots")
        .iter()
        .map(|root| root.as_str().expect("root").to_owned())
        .collect::<Vec<_>>();
    assert_eq!(roots, ROOTS);
    let frozen_paths = model::original(&root);
    let analysis = crate::repository_files::analyze(&root, &model::policies())
        .expect("complete frozen inspection");
    assert!(analysis.findings.is_empty());
    let native_paths = analysis
        .policies
        .iter()
        .filter(|rule| rule.policy_id.ends_with("kd-source-traversal"))
        .flat_map(|rule| &rule.entries)
        .filter(|entry| {
            entry.kind != "directory"
                && Path::new(&entry.path)
                    .extension()
                    .is_some_and(|ext| ext == "rs")
        })
        .map(|entry| entry.path.clone())
        .collect::<Vec<_>>();
    assert_eq!(native_paths, frozen_paths);
    let frozen_inputs = frozen_paths
        .iter()
        .map(|path| (path.clone(), hash(&root.join(path))))
        .collect();
    let mut physical = super::portable();
    physical.extend(super::unix::links_and_specials());
    physical.extend(super::unix::permissions());
    assert_eq!(physical.len(), 127);
    let injected = super::injected::run();
    let pins: serde_json::Value = serde_json::from_slice(
        &fs::read(project.join("crates/zrail-testkit/tests/fixtures/rc9/snapshots.json"))
            .expect("pins"),
    )
    .expect("pin JSON");
    let snapshot = pins["consumers"]
        .as_array()
        .expect("pins")
        .iter()
        .find(|row| row["name"] == "kafka-driver")
        .expect("snapshot")
        .clone();
    let rustc = Command::new("rustc").arg("-Vv").output().expect("compiler");
    assert!(rustc.status.success());
    let report = Report {
        schema: 1,
        implementation_commit: commit.clone(),
        implementation_tree: git(&project, &["rev-parse", "HEAD^{tree}"]),
        test_binary_sha256: hash(&std::env::current_exe().expect("binary")),
        rustc_version: String::from_utf8(rustc.stdout)
            .expect("compiler identity")
            .trim()
            .into(),
        cargo_lock_sha256: hash(&project.join("Cargo.lock")),
        policy_sha256: hash(&project.join(POLICY)),
        snapshot,
        full_repository_qualified: false,
        guardrails_sha256,
        support_sha256,
        roots,
        frozen_inputs,
        frozen_paths,
        frozen_observations: analysis.policies,
        physical,
        injected,
        limitations: [
            "Only three physical traversal preconditions over six frozen roots; no complete source/Cargo policy, root configuration migration, or downstream cutover.",
            "Unread pruned/link descendants and unselected unread directories fail closed more strongly than the old selected-root traversal.",
            "The old walker does not recurse directory links; its link-cycle cases terminate and are executed. Special-entry observations never read content streams.",
            "Injected original failure expressions and native scan errors are separate from real OS permission cases; native metadata inspection is not DirEntry::file_type.",
        ],
    };
    verify();
    assert_eq!(git(&project, &["rev-parse", "HEAD"]), commit);
    let mut payload =
        serde_json::to_vec_pretty(&serde_json::to_value(report).expect("typed canonical report"))
            .expect("JSON");
    payload.push(b'\n');
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .expect("new report")
        .write_all(&payload)
        .expect("write report");
}

#[cfg(unix)]
fn git(root: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()
        .expect("Git");
    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .expect("Git UTF-8")
        .trim()
        .into()
}

#[cfg(not(unix))]
pub(super) fn run() {
    panic!("qualification requires a Unix host with enforced permissions");
}
