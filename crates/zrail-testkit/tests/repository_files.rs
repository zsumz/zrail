//! Physical file policies enforce exact quantities, raw predicates, and immutable inputs.

#[path = "repository_files/boundary_test.rs"]
mod boundary_test;
#[path = "repository_files/encoding_test.rs"]
mod encoding_test;
#[path = "strict_facades/fixture.rs"]
mod fixture;
#[path = "repository_files/line_test.rs"]
mod line_test;
#[path = "repository_files/prefix_test.rs"]
mod prefix_test;
#[path = "repository_files/support.rs"]
mod support;
#[path = "repository_files/text_structure_test.rs"]
mod text_structure_test;

use std::fs;
use support::{configured, coverage, violation};
use zrail_core::ReportStatus;

#[test]
fn required_paths_fail_when_missing_and_zero_counts_remain_active() {
    let repository = configured(
        r#"
[[repository.files]]
name = "license"
include = ["LICENSE"]
reason = "Required root license."
predicate = { kind = "count", minimum = 1, maximum = 1 }
[[repository.files]]
name = "retired"
include = ["old/scanner.py"]
entry = "any"
reason = "Retired evaluators must not return."
predicate = { kind = "count", minimum = 0, maximum = 0 }
"#,
    );
    repository.lock();
    violation(&repository, "license", "REP-FILE-001");
    repository.write("LICENSE", "license\n");
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    assert_eq!(repository.explain("src/child.rs").design_target, Some(240));
    fs::create_dir(repository.0.join("old")).expect("retired parent");
    repository.write("old/scanner.py", "# a returned evaluator\n");
    violation(&repository, "retired", "REP-FILE-001");
    fs::remove_file(repository.0.join("old/scanner.py")).expect("remove retired file");
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    fs::remove_file(repository.0.join("LICENSE")).expect("remove required file");
    violation(&repository, "license", "REP-FILE-001");
}

#[test]
fn exact_sets_reject_exchanging_members_even_with_an_unchanged_count() {
    let repository = configured(
        r#"
[[repository.files]]
name = "fixtures"
include = ["data/*.json", "data/one.json"]
reason = "Every exact fixture identity matters."
predicate = { kind = "exact-paths", paths = ["data/one.json", "data/two.json"] }
"#,
    );
    fs::create_dir(repository.0.join("data")).expect("fixture directory");
    repository.write("data/one.json", "{}");
    repository.write("data/two.json", "{}");
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    assert_eq!(coverage(&repository).repository_files[0].entries.len(), 2);
    fs::rename(
        repository.0.join("data/two.json"),
        repository.0.join("data/three.json"),
    )
    .expect("exchange fixture identity");
    violation(&repository, "fixtures", "REP-FILE-002");
    assert_eq!(coverage(&repository).repository_files[0].entries.len(), 2);
}

#[test]
fn name_parts_case_and_filesystem_context_are_explicit() {
    let repository = configured(
        r#"
[[repository.files]]
name = "names"
include = ["assets/**/*.txt"]
reason = "Directory stems participate in the legacy name policy."
predicate = { kind = "forbidden-names", names = ["helpers"], part = "component-stem", case = "ascii-insensitive" }
"#,
    );
    fs::create_dir_all(repository.0.join("assets/Helpers.rs")).expect("named directory");
    repository.write("assets/Helpers.rs/data.txt", "payload");
    violation(&repository, "names", "REP-FILE-003");
    let observed = coverage(&repository);
    assert_eq!(
        observed.repository_files[0].entries[0].forbidden_names,
        ["Helpers"]
    );
    assert_eq!(observed.repository_files[0].checkout_path, None);
    let source = fs::read_to_string(repository.0.join("zrail.toml")).expect("contract");
    let name = repository
        .0
        .file_name()
        .expect("checkout name")
        .to_str()
        .expect("UTF-8");
    repository.write(
        "zrail.toml",
        &source.replace(
            "names = [\"helpers\"]",
            &format!("names = [\"{name}\"], basis = 'filesystem'"),
        ),
    );
    violation(&repository, "names", "REP-FILE-003");
    assert_eq!(
        coverage(&repository).repository_files[0]
            .checkout_path
            .as_deref(),
        repository
            .0
            .canonicalize()
            .expect("canonical checkout path")
            .to_str()
    );
}

#[test]
fn raw_prefix_predicates_do_not_accept_matching_text_in_an_irrelevant_position() {
    let repository = configured(
        r#"
[[repository.files]]
name = "leading-doc"
include = ["src/child.rs"]
reason = "Preserve the deliberately raw leading documentation marker."
predicate = { kind = "literal", text = "//!", mode = "starts-with", normalization = "trim-start" }
"#,
    );
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    repository.write(
        "src/child.rs",
        "#[doc = \"//! misleading\"]\npub struct State;\n",
    );
    violation(&repository, "leading-doc", "REP-FILE-004");
    repository.write("src/child.rs", "\u{2003}//! Data.\npub struct State;\n");
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    let explanation = repository.explain("src/child.rs");
    assert_eq!(explanation.repository_files[0].claim, "raw-utf8-text");
    assert!(explanation.human().contains("repository:file:leading-doc"));
}

#[test]
fn raw_counts_are_nonoverlapping_and_samples_never_change_totals() {
    let repository = configured(
        r#"
[[repository.files]]
name = "count"
include = ["script"]
reason = "Exact written occurrence count."
predicate = { kind = "literal", text = "aa", mode = "exact-count", count = 20 }
"#,
    );
    repository.write("script", &"a".repeat(40));
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    let observed = coverage(&repository);
    let entry = &observed.repository_files[0].entries[0];
    assert_eq!(entry.literal_count, Some(20));
    assert_eq!(entry.literal_offsets.len(), 16);
    assert_eq!(entry.omitted_literal_offsets, 4);
    repository.write("script", &"a".repeat(42));
    violation(&repository, "count", "REP-FILE-004");
    fs::remove_file(repository.0.join("script")).expect("missing positive subject");
    violation(&repository, "count", "REP-FILE-004");
}

#[test]
fn forbidden_literals_accept_zero_files_and_include_comments_as_authored() {
    let repository = configured(
        r#"
[[repository.files]]
name = "vocabulary"
include = ["notes/*.txt"]
reason = "This is a deliberate text policy, including comments."
predicate = { kind = "literal", text = "retired_api", mode = "absent", normalization = "remove-whitespace" }
"#,
    );
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    fs::create_dir(repository.0.join("notes")).expect("notes");
    repository.write("notes/a.txt", "// retired_\napi in a comment\n");
    violation(&repository, "vocabulary", "REP-FILE-004");
    repository.write("notes/a.txt", "RETIRED_API");
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
}

#[test]
fn byte_equality_handles_binary_inputs_and_requires_both_sides() {
    let repository = configured(
        r#"
[[repository.files]]
name = "license-copy"
include = ["COPY"]
reason = "Exact bytes, without UTF-8 or whitespace normalization."
predicate = { kind = "bytes-equal", other = "LICENSE" }
"#,
    );
    let bytes = [0xff, 0, b'\r', b'\n'];
    fs::write(repository.0.join("LICENSE"), bytes).expect("reference bytes");
    fs::write(repository.0.join("COPY"), bytes).expect("copy bytes");
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    assert!(repository.explain("LICENSE").repository_files[0].is_reference);
    repository.write("COPY", "different");
    violation(&repository, "license-copy", "REP-FILE-005");
    fs::remove_file(repository.0.join("COPY")).expect("remove copy");
    violation(&repository, "license-copy", "REP-FILE-005");
    repository.write("COPY", "present");
    fs::remove_file(repository.0.join("LICENSE")).expect("remove reference");
    violation(&repository, "license-copy", "REP-FILE-005");
}

#[test]
fn relevant_bytes_and_new_selected_paths_invalidate_existing_input_binding() {
    let repository = configured(
        r#"
[[repository.files]]
name = "marker"
include = ["data/*.txt"]
reason = "Lock the reviewed raw inputs as well as their positive predicate."
predicate = { kind = "literal", text = "marker", mode = "contains" }
"#,
    );
    fs::create_dir(repository.0.join("data")).expect("data directory");
    repository.write("data/a.txt", "marker one");
    repository.lock();
    let first = repository.check().candidate_lock.expect("complete lock");
    repository.write("data/a.txt", "marker two");
    let result = repository.check();
    assert!(
        result
            .report
            .findings
            .iter()
            .any(|finding| finding.id == "LOCK-028")
    );
    let second = result.candidate_lock.expect("complete changed input");
    assert_ne!(first.analysis, second.analysis);
    repository.write("data/b.txt", "marker two");
    let third = repository
        .check()
        .candidate_lock
        .expect("complete new path");
    assert_ne!(second.analysis, third.analysis);
    let report = coverage(&repository);
    assert!(report.repository_files[0].satisfied);
    assert_eq!(
        report.json().expect("first serialization"),
        coverage(&repository).json().expect("repeat serialization")
    );
}
