//! Trusted pinned-checkout traversal includes every tracked path and nested workspace.

use std::{fs, path::Path, process::Command};

use serde::Serialize;
use syn::visit::Visit;

use super::collector::{Candidate, Collector, Function};

#[derive(Debug, Serialize)]
pub(super) struct FileRecord {
    pub(super) repository: String,
    pub(super) commit: String,
    pub(super) path: String,
    pub(super) mode: String,
    pub(super) git_object: String,
    pub(super) sha256: Option<String>,
    pub(super) parse_error: Option<String>,
    pub(super) functions: Vec<Function>,
    pub(super) candidates: Vec<Candidate>,
    pub(super) review: &'static str,
}

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .expect("run trusted Git inspection");
    assert!(
        output.status.success(),
        "Git inspection failed: {:?}",
        output.stderr
    );
    String::from_utf8(output.stdout).expect("UTF-8 snapshot paths")
}

pub(super) fn inspect(
    root: &Path,
    pin: &serde_json::Value,
    additional: &super::inputs::AdditionalInputs,
) -> Vec<FileRecord> {
    let repository = pin["repository"].as_str().expect("repository pin");
    let commit = pin["commit"].as_str().expect("commit pin");
    assert_eq!(git(root, &["rev-parse", "HEAD"]).trim(), commit);
    assert_eq!(
        git(root, &["rev-parse", "HEAD^{tree}"]).trim(),
        pin["tree"].as_str().expect("tree pin")
    );
    assert!(
        git(root, &["status", "--porcelain=v1", "--untracked-files=all"]).is_empty(),
        "dirty snapshot"
    );
    let tree = git(root, &["ls-tree", "-rz", "--full-tree", "HEAD"]);
    assert!(
        tree.len() <= 32 * 1024 * 1024,
        "snapshot tree exceeds census bound"
    );
    let mut output = Vec::new();
    for entry in tree.split('\0').filter(|entry| !entry.is_empty()) {
        let (header, path) = entry.split_once('\t').expect("Git tree entry");
        let header = header.split_whitespace().collect::<Vec<_>>();
        let mut record = FileRecord {
            repository: repository.into(),
            commit: commit.into(),
            path: path.into(),
            mode: header[0].into(),
            git_object: header[2].into(),
            sha256: None,
            parse_error: None,
            functions: Vec::new(),
            candidates: Vec::new(),
            review: "pending",
        };
        if !matches!(header[0], "100644" | "100755") {
            record.parse_error =
                Some("non-regular Git entry requires separate source review".into());
            output.push(record);
            continue;
        }
        let full = root.join(path);
        let metadata = fs::symlink_metadata(&full).expect("snapshot input metadata");
        assert!(
            metadata.is_file() && metadata.len() <= 32 * 1024 * 1024,
            "non-regular or oversized input: {path}"
        );
        let bytes = fs::read(&full).expect("read frozen file");
        record.sha256 = Some(zrail_core::sha256_hex(&bytes));
        let explicit = additional.selected(
            repository,
            commit,
            path,
            record.sha256.as_deref().expect("regular input hash"),
        );
        if explicit
            || Path::new(path)
                .extension()
                .is_some_and(|value| value == "rs")
        {
            let source = String::from_utf8(bytes).expect("UTF-8 Rust input");
            match syn::parse_file(&source) {
                Ok(syntax) => {
                    let mut collector = Collector::default();
                    collector.prefix = format!("{repository}:{path}");
                    collector.visit_file(&syntax);
                    record.functions = collector.functions;
                    record.candidates = collector.candidates;
                }
                Err(error) => record.parse_error = Some(error.to_string()),
            }
        }
        output.push(record);
    }
    output.sort_by(|left, right| left.path.cmp(&right.path));
    assert!(
        git(root, &["status", "--porcelain=v1", "--untracked-files=all"]).is_empty(),
        "snapshot changed during census"
    );
    output
}
