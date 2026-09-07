//! Frozen raw boundaries compare with native typed policies, independently of Rust reachability.

#[path = "legacy.rs"]
mod legacy;

use std::{collections::BTreeSet, fs, path::PathBuf};

use serde::Deserialize;
use zrail_core::{RepositoryFilePredicate, RepositoryFileRule};

use super::model::project;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fragment {
    repository: Rules,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rules {
    files: Vec<RepositoryFileRule>,
}

fn policies() -> Vec<RepositoryFileRule> {
    let fragment: Fragment = toml::from_str(
        &fs::read_to_string(
            project().join("docs/rc9/policies/rafter.source-boundaries.fragment.toml"),
        )
        .expect("reviewed boundary policy"),
    )
    .expect("strict stock file rules");
    assert_eq!(fragment.repository.files.len(), 158);
    fragment.repository.files
}

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir()
            .canonicalize()
            .expect("canonical temporary directory")
            .join(format!(
                "zrail-boundary-parity-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
        fs::create_dir(&root).expect("fresh isolated fixture");
        Self(root)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove only this test's fixture");
    }
}

#[test]
fn frozen_boundary_tokens_preserve_raw_prefix_end_and_qualified_path_semantics() {
    let policies = policies();
    let fixture = Fixture::new();
    let mut instances = 0;
    for policy in policies.iter().filter(|rule| rule.name.ends_with("-path")) {
        let RepositoryFilePredicate::Literal(literal) = &policy.predicate else {
            panic!("path literal")
        };
        let token = literal.text.strip_suffix("::").expect("qualified marker");
        let prefix = policy.name.strip_suffix("-path").unwrap();
        let rules: Vec<_> = policies
            .iter()
            .filter(|rule| rule.name.starts_with(&format!("{prefix}-")))
            .cloned()
            .collect();
        assert_eq!(rules.len(), 5);
        let path = policy.include[0].replace("/**/*.rs", "/nested/unit_test.rs");
        fs::create_dir_all(fixture.0.join(&path).parent().unwrap()).unwrap();
        for leading in ["use ", "pub use ", "extern crate "] {
            for boundary in ["", ":", ";", ",", "{", " ", "\t"] {
                let source = format!("\u{2003}{leading}{token}{boundary}\r\n");
                fs::write(fixture.0.join(&path), &source).unwrap();
                assert!(!legacy::accepts(&source, token));
                let analysis = crate::repository_files::analyze(&fixture.0, &rules).unwrap();
                let suffix = if boundary.is_empty() {
                    match leading {
                        "use " => "end-1",
                        "pub use " => "end-2",
                        _ => "end-3",
                    }
                } else {
                    "prefixes"
                };
                let diagnostic = if boundary.is_empty() {
                    "REP-FILE-004"
                } else {
                    "REP-FILE-008"
                };
                assert_eq!(analysis.findings.len(), 1, "{source:?}");
                assert_eq!(
                    analysis.findings[0].rule,
                    format!("repository:file:{prefix}-{suffix}")
                );
                assert_eq!(analysis.findings[0].id, diagnostic);
            }
        }
        for source in [
            String::new(),
            format!("use {token}_helpers;"),
            format!("// use {token};"),
            format!("let s = \"use {token};\";"),
            format!("pub(crate) use {token};"),
            format!("use {{{token}}};"),
            format!("use {token}\r"),
            format!("mod nested {{ use {token}; }}"),
        ] {
            fs::write(fixture.0.join(&path), &source).unwrap();
            assert!(legacy::accepts(&source, token), "{source:?}");
            assert!(
                crate::repository_files::analyze(&fixture.0, &rules)
                    .unwrap()
                    .findings
                    .is_empty(),
                "{source:?}"
            );
        }
        for source in [
            format!("// {token}::"),
            format!("const S: &str = \"{token}::\";"),
            format!("#[cfg(any())]\nfn disabled() {{ {token}::run(); }}"),
        ] {
            fs::write(fixture.0.join(&path), &source).unwrap();
            assert!(!legacy::accepts(&source, token));
            let analysis = crate::repository_files::analyze(&fixture.0, &rules).unwrap();
            assert_eq!(analysis.findings.len(), 1);
            assert_eq!(
                analysis.findings[0].rule,
                format!("repository:file:{prefix}-path")
            );
            assert_eq!(analysis.findings[0].id, "REP-FILE-004");
        }
        fs::remove_file(fixture.0.join(&path)).unwrap();
        instances += 1;
    }
    assert_eq!(instances, 31);
}

#[test]
fn boundary_roots_remain_required_while_empty_trees_and_absent_tokens_are_valid() {
    let rules = policies();
    let fixture = Fixture::new();
    let analysis = crate::repository_files::analyze(&fixture.0, &rules).unwrap();
    assert_eq!(analysis.findings.len(), 3);
    for finding in analysis.findings {
        assert_eq!(finding.id, "REP-FILE-001");
        assert!(
            finding
                .rule
                .starts_with("repository:file:rf-boundary-root-")
        );
    }
    for root in ["rafter", "rafter-app", "rafter-runtime-api"] {
        fs::create_dir_all(fixture.0.join(format!("crates/{root}/src"))).unwrap();
    }
    assert!(
        crate::repository_files::analyze(&fixture.0, &rules)
            .unwrap()
            .findings
            .is_empty()
    );
    for root in ["rafter", "rafter-app", "rafter-runtime-api"] {
        let path = fixture.0.join(format!("crates/{root}/src/new.rs"));
        fs::write(&path, [0xff]).unwrap();
        let error = crate::repository_files::analyze(&fixture.0, &rules).unwrap_err();
        assert!(
            error.starts_with(&format!(
                "REP-FILE-006: incomplete repository-file analysis: repository:file:rf-boundary-{root}-"
            )),
            "{error}"
        );
        fs::remove_file(path).unwrap();
    }
}

#[test]
#[ignore = "requires explicitly prefetched ZRAIL_RC9_SNAPSHOTS; slice acceptance only"]
fn frozen_rafter_boundary_files_fit_the_work_bound_and_match_the_original() {
    let snapshots =
        PathBuf::from(std::env::var_os("ZRAIL_RC9_SNAPSHOTS").expect("prefetched snapshots"));
    assert!(
        std::process::Command::new("python3")
            .arg(project().join("scripts/rc9-snapshots"))
            .arg(&snapshots)
            .status()
            .unwrap()
            .success()
    );
    let root = snapshots.join("rafter");
    let rules = policies();
    let analysis =
        crate::repository_files::analyze(&root, &rules).expect("complete bounded file slice");
    assert!(analysis.findings.is_empty(), "{:?}", analysis.findings);
    let mut inputs = BTreeSet::new();
    for policy in analysis
        .policies
        .iter()
        .filter(|policy| policy.policy.name.ends_with("-path"))
    {
        let RepositoryFilePredicate::Literal(literal) = &policy.policy.predicate else {
            panic!("path literal")
        };
        let token = literal.text.strip_suffix("::").unwrap();
        assert!(!policy.entries.is_empty());
        for entry in &policy.entries {
            let source = fs::read_to_string(root.join(&entry.path)).unwrap();
            assert!(legacy::accepts(&source, token), "{}: {token}", entry.path);
            assert_eq!(
                entry.sha256,
                Some(zrail_core::sha256_hex(source.as_bytes()))
            );
            inputs.insert(entry.path.clone());
        }
    }
    assert_eq!(inputs.len(), 194);
}
