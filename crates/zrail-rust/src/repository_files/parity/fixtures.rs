//! Mutations run the stock file analyzer on isolated physical fixtures with original selectors.

use std::{
    fs,
    path::{Path, PathBuf},
};

use zrail_core::{RepositoryFilePredicate, RepositoryFileRule, sha256_hex};

use super::{legacy, model::FixtureOutcome};

pub(super) fn qualify(root: &Path, policies: &[RepositoryFileRule]) -> Vec<FixtureOutcome> {
    let mut rows = Vec::new();
    for policy in policies {
        let fixture = Fixture::new(root);
        match &policy.predicate {
            RepositoryFilePredicate::Count { .. } => {
                let path = &policy.include[0];
                rows.push(check(root, policy, path, None, false));
                fs::create_dir_all(root.join(path)).expect("required directory");
                rows.push(check(root, policy, path, None, true));
                fs::remove_dir(root.join(path)).expect("remove required directory");
                fs::write(root.join(path), "ordinary file").expect("wrong entry kind");
                rows.push(check(root, policy, path, Some("ordinary file"), false));
            }
            RepositoryFilePredicate::ForbiddenNames { .. } => {
                for path in [
                    "src/identity/mod.rs",
                    "src/Helpers/mod.rs",
                    "src/common/mod.rs",
                    "src/context.rs/owned.rs",
                    "src/helpers/mod.rs",
                    "src/helpers.rs/owned.rs",
                    "src/manager/owned.rs",
                    "src/utils/mod.rs",
                ] {
                    fixture.write(path, "//! A written module.\n");
                    let accepted = legacy::names(&root.join(path));
                    rows.push(check(
                        root,
                        policy,
                        path,
                        Some("//! A written module.\n"),
                        accepted,
                    ));
                    fs::remove_file(root.join(path)).expect("remove isolated case");
                }
            }
            RepositoryFilePredicate::Literal(literal) => {
                let path = if policy.name.starts_with("kd-capability-") {
                    format!(
                        "{}/rc9_fixture.rs",
                        policy.include[0].trim_end_matches("/**/*.rs")
                    )
                } else {
                    "src/rc9_fixture.rs".into()
                };
                let mut sources = if policy.name.starts_with("kd-capability-") {
                    vec![
                        "//! Owns a deterministic value.\n".into(),
                        format!("//! Owns a value.\n// {}\n", literal.text),
                        format!("const TEXT: &str = {:?};\n", literal.text),
                        format!("use {} as Renamed;\n", literal.text),
                        format!("use {} as Renamed;\n", literal.text.replace("::", " :: ")),
                    ]
                } else {
                    vec![
                        "//! Owns a value.\n".into(),
                        "//! Owns one concept.\npub struct Owned;".into(),
                        " \n\t//! Owns a value.\n".into(),
                        "pub struct Unowned;".into(),
                        "#[doc = \"//! elsewhere\"] pub struct Unowned;".into(),
                    ]
                };
                if literal.text == "std::net" {
                    sources.push("use std::net::TcpStream;".into());
                }
                for source in sources {
                    fixture.write(&path, &source);
                    let accepted = if policy.name.starts_with("kd-capability-") {
                        legacy::capability(&path, &source, &literal.text)
                    } else {
                        legacy::contract(&source)
                    };
                    rows.push(check(root, policy, &path, Some(&source), accepted));
                }
            }
            _ => panic!("unreviewed frozen fixture family"),
        }
    }
    rows
}

fn check(
    root: &Path,
    rule: &RepositoryFileRule,
    path: &str,
    source: Option<&str>,
    accepted: bool,
) -> FixtureOutcome {
    let analysis = super::super::analyze(root, std::slice::from_ref(rule))
        .expect("complete isolated file analysis");
    let native = analysis.findings.is_empty();
    assert_eq!(
        native, accepted,
        "{} {path}: {:?}",
        rule.name, analysis.findings
    );
    let expected = match rule.predicate {
        RepositoryFilePredicate::Count { .. } => "REP-FILE-001",
        RepositoryFilePredicate::ForbiddenNames { .. } => "REP-FILE-003",
        _ => "REP-FILE-004",
    };
    let diagnostic = (!accepted).then(|| {
        assert_eq!(analysis.findings.len(), 1);
        assert_eq!(
            analysis.findings[0].rule,
            format!("repository:file:{}", rule.name)
        );
        assert_eq!(analysis.findings[0].id, expected);
        expected.into()
    });
    FixtureOutcome {
        policy_id: format!("repository:file:{}", rule.name),
        path: path.into(),
        entry_kind: if root.join(path).is_dir() {
            "directory"
        } else if root.join(path).is_file() {
            "file"
        } else {
            "missing"
        }
        .into(),
        source_sha256: source.map(|source| sha256_hex(source.as_bytes())),
        legacy_accepted: accepted,
        native_accepted: native,
        diagnostic,
    }
}

struct Fixture(PathBuf);

impl Fixture {
    fn new(root: &Path) -> Self {
        fs::create_dir(root).expect("new isolated fixture; existing state is never reset");
        Self(root.into())
    }

    fn write(&self, path: &str, source: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().expect("fixture parent")).expect("fixture directories");
        fs::write(path, source).expect("fixture source");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove test-owned fixture");
    }
}
