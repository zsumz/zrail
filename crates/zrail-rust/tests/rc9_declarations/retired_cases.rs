//! Independent expected diagnostics prevent an unrelated failure from masquerading as parity.

use super::model::{self, BACKEND, CONSTRUCTION, Fixture, Fragment, MODULES, NAMES, Outcome};
use std::fs;

pub(super) fn run(fixture: &Fixture, policy: &Fragment) -> Vec<Outcome> {
    let baseline = fixture.hashes();
    let mut rows = vec![check(fixture, policy, "valid", None)];
    for name in NAMES {
        for (case, source, rejected) in [
            ("external", format!("mod {name};"), true),
            ("inline", format!("pub mod {name} {{}}"), true),
            ("inactive", format!("#[cfg(any())] mod {name};"), true),
            ("duplicate", format!("mod {name}; mod {name};"), true),
            ("nested", format!("mod child {{ mod {name}; }}"), false),
            ("raw", format!("mod r#{name};"), false),
            (
                "opaque",
                format!("macro_rules! hidden {{ () => {{ mod {name}; }}; }}"),
                false,
            ),
            ("comment", format!("// mod {name};"), false),
        ] {
            fixture.write(MODULES, source);
            rows.push(check(
                fixture,
                policy,
                &format!("module-{name}-{case}"),
                rejected.then_some(("rust:inventory:kd-retired-modules", "RUST-INVENTORY-001")),
            ));
        }
        fixture.write(MODULES, &fixture.inputs[MODULES]);
        for (case, path, rejected) in [
            ("direct", "old.rs", true),
            ("nested", "a/b/old.rs", true),
            ("uppercase", "old.RS", false),
            ("extensionless", ".rs", false),
            ("nested-extensionless", "a/.rs", false),
            ("hidden", ".old.rs", true),
            ("text", "old.txt", false),
        ] {
            let path = format!("src/reactor/{name}/{path}");
            fixture.write(&path, "// retired physical source\n");
            let identity = format!("repository:file:kd-retired-tree-{name}");
            rows.push(check(
                fixture,
                policy,
                &format!("tree-{name}-{case}"),
                rejected.then_some((&identity, "REP-FILE-001")),
            ));
            fs::remove_dir_all(fixture.root.join("src/reactor").join(name))
                .expect("remove test-owned tree");
        }
        fs::create_dir_all(fixture.root.join("src/reactor").join(name).join("empty.rs"))
            .expect("empty directory");
        rows.push(check(
            fixture,
            policy,
            &format!("tree-{name}-empty-directory"),
            None,
        ));
        fs::remove_dir_all(fixture.root.join("src/reactor").join(name))
            .expect("remove test-owned tree");
    }
    for (case, source, rejected) in [
        ("unit", "enum ReactorBackend { Legacy }", true),
        ("tuple", "enum ReactorBackend { Legacy(u8) }", true),
        ("named", "enum ReactorBackend { Legacy { x: u8 } }", true),
        (
            "inactive",
            "enum ReactorBackend { #[cfg(any())] Legacy }",
            true,
        ),
        ("duplicate", "enum ReactorBackend { Legacy, Legacy }", true),
        ("different-enum", "enum Other { Legacy }", false),
        (
            "nested",
            "mod child { enum ReactorBackend { Legacy } }",
            false,
        ),
        ("raw", "enum ReactorBackend { r#Legacy }", false),
        ("absent-enum", "", false),
        ("comment", "// enum ReactorBackend { Legacy }", false),
    ] {
        fixture.write(BACKEND, source);
        rows.push(check(
            fixture,
            policy,
            &format!("backend-{case}"),
            rejected.then_some(("rust:inventory:kd-retired-backend", "RUST-INVENTORY-001")),
        ));
    }
    fixture.write(BACKEND, &fixture.inputs[BACKEND]);
    for token in ["new_legacy", "LegacyBackend"] {
        for (case, source, rejected) in [
            ("comment", format!("// {token}"), true),
            ("string", format!("const TEXT: &str = \"{token}\";"), true),
            ("inactive", format!("#[cfg(any())] fn {token}() {{}}"), true),
            ("substring", format!("// prefix{token}suffix"), true),
            (
                "different-case",
                format!("// {}", token.to_uppercase()),
                false,
            ),
        ] {
            fixture.write(CONSTRUCTION, source);
            let identity = format!("repository:file:kd-retired-construction-{token}");
            rows.push(check(
                fixture,
                policy,
                &format!("construction-{token}-{case}"),
                rejected.then_some((&identity, "REP-FILE-004")),
            ));
        }
    }
    fixture.write(CONSTRUCTION, &fixture.inputs[CONSTRUCTION]);
    for (name, path, identity) in [
        ("modules", MODULES, "rust:inventory:kd-retired-modules"),
        ("backend", BACKEND, "rust:inventory:kd-retired-backend"),
        ("construction", CONSTRUCTION, "repository:files"),
    ] {
        fs::remove_file(fixture.root.join(path)).expect("remove test-owned required file");
        rows.push(check(
            fixture,
            policy,
            &format!("input-{name}-missing"),
            Some(("repository:file:kd-retired-inputs", "REP-FILE-002")),
        ));
        fixture.write(path, [0xff]);
        rows.push(check(
            fixture,
            policy,
            &format!("input-{name}-utf8"),
            Some((
                identity,
                if path == CONSTRUCTION {
                    "REP-FILE-006"
                } else {
                    "RUST-INVENTORY-002"
                },
            )),
        ));
        if path != CONSTRUCTION {
            for (case, source) in [("malformed", "fn broken("), ("fragment", "1 + 2")] {
                fixture.write(path, source);
                rows.push(check(
                    fixture,
                    policy,
                    &format!("input-{name}-{case}"),
                    Some((identity, "RUST-INVENTORY-002")),
                ));
            }
        }
        fixture.write(path, &fixture.inputs[path]);
    }
    fixture.write(
        "src/unrelated.rs",
        "mod tls; enum ReactorBackend { Legacy } // LegacyBackend new_legacy",
    );
    rows.push(check(fixture, policy, "wrong-file-decoy", None));
    fs::remove_file(fixture.root.join("src/unrelated.rs")).expect("remove only decoy");
    assert_eq!(
        fixture.hashes(),
        baseline,
        "restore each independent fixture"
    );
    rows
}

fn check(
    fixture: &Fixture,
    policy: &Fragment,
    case: &str,
    expected: Option<(&str, &str)>,
) -> Outcome {
    let row = model::observe(fixture, policy, case);
    assert_eq!(
        row.native_accepted,
        expected.is_none(),
        "independent acceptance: {case}"
    );
    let expected = expected
        .map(|(policy, id)| (policy.to_owned(), id.to_owned()))
        .into_iter()
        .collect::<Vec<_>>();
    assert_eq!(row.diagnostics, expected, "intended diagnostic: {case}");
    row
}
