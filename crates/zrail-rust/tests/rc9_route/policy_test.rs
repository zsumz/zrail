//! Mixed-file source assertions migrate independently of the retained runtime scenario.

#[path = "compiler.rs"]
mod compiler;
#[path = "fixtures.rs"]
mod fixtures;
#[path = "legacy.rs"]
mod legacy;
#[path = "model.rs"]
mod model;
#[path = "qualification.rs"]
mod qualification;
#[path = "report.rs"]
mod report;

use std::fs;

use model::{FACADE, FORBIDDEN, Fixture, MARKER};

#[test]
#[ignore = "requires prefetched snapshots, a committed tree and a fresh ZRAIL_RC9_ROUTE_REPORT"]
fn qualify_frozen_route_source_and_compile_input_parity() {
    qualification::run();
}

#[test]
fn route_bans_preserve_every_selected_file_and_raw_matching_context() {
    let policies = model::policies();
    let fixture = Fixture::new("bans", &policies);
    check(&fixture, &policies, None);
    for (path, original) in &fixture.inputs {
        if path == FACADE {
            continue;
        }
        for token in FORBIDDEN {
            let identity = format!("repository:file:kd-route-forbid-{token}");
            for addition in [
                format!("\n// {token}\n"),
                format!("\nconst TEXT: &str = \"{token}\";\n"),
                format!("\nuse crate::{token} as Renamed;\n"),
            ] {
                fixture.write(path, format!("{original}{addition}"));
                check(&fixture, &policies, Some((&identity, "REP-FILE-004")));
            }
            fixture.write(path, format!("{original}\n// {}\n", token.to_lowercase()));
            check(&fixture, &policies, None);
        }
        fixture.write(path, original);
    }
    fixture.write("unrelated.rs", FORBIDDEN.join(" "));
    check(&fixture, &policies, None);
}

#[test]
fn route_facade_preserves_exact_raw_quantity_and_file_location() {
    let policies = model::policies();
    let fixture = Fixture::new("facade", &policies);
    let original = &fixture.inputs[FACADE];
    let removed = original.replace(MARKER, "connections: Other<T>");
    for source in [
        removed.clone(),
        format!("{original}\n// {MARKER}\n"),
        original.replace(MARKER, "connections : DirectSetOwner<T>"),
    ] {
        fixture.write(FACADE, source);
        check(
            &fixture,
            &policies,
            Some(("repository:file:kd-route-owner-field", "REP-FILE-004")),
        );
    }
    fixture.write("unrelated.rs", MARKER);
    fixture.write(FACADE, &removed);
    check(
        &fixture,
        &policies,
        Some(("repository:file:kd-route-owner-field", "REP-FILE-004")),
    );
    // The original policy deliberately counts comments, without semantic field identity.
    fixture.write(FACADE, format!("{removed}\n// {MARKER}\n"));
    check(&fixture, &policies, None);
}

#[test]
fn route_inputs_remain_required_when_absence_predicates_have_zero_matches() {
    let policies = model::policies();
    let fixture = Fixture::new("inputs", &policies);
    for (path, original) in &fixture.inputs {
        fs::remove_file(fixture.root.join(path)).expect("delete selected input");
        let analysis = crate::repository_files::analyze(&fixture.root, &policies)
            .expect("complete absent-path inventory");
        assert!(!fixture.original_accepts());
        assert!(analysis.findings.iter().any(|finding| {
            finding.rule == "repository:file:kd-route-inputs" && finding.id == "REP-FILE-002"
        }));
        fixture.write(path, [0xff]);
        assert!(!fixture.original_accepts());
        let error = crate::repository_files::analyze(&fixture.root, &policies)
            .expect_err("invalid UTF-8 cannot satisfy the original include_str contract");
        assert!(error.starts_with("REP-FILE-006:"), "{error}");
        fixture.write(path, original);
    }
    check(&fixture, &policies, None);
}

fn check(
    fixture: &Fixture,
    policies: &[zrail_core::RepositoryFileRule],
    expected: Option<(&str, &str)>,
) {
    let analysis = crate::repository_files::analyze(&fixture.root, policies)
        .expect("complete bounded physical file analysis");
    assert_eq!(fixture.original_accepts(), expected.is_none());
    let findings = analysis
        .findings
        .iter()
        .map(|finding| (finding.rule.as_str(), finding.id.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(findings, expected.into_iter().collect::<Vec<_>>());
    for policy in analysis.policies {
        assert_eq!(policy.analysis, zrail_core::AnalysisQuality::Exact);
        assert!(matches!(
            policy.claim.as_str(),
            "raw-utf8-text" | "physical-paths"
        ));
    }
}
