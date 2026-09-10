//! Omitted subtrees, links, and unreadable bytes cannot become false absence evidence.

use super::support::{configured, coverage, violation};
use std::fs;

#[test]
fn source_exclusions_do_not_exclude_independently_governed_raw_files() {
    let repository = configured(
        r#"
[[repository.files]]
name = "raw"
include = ["assets/**/*.txt"]
reason = "Excluded source is still subject to explicit raw policy."
predicate = { kind = "literal", text = "forbidden", mode = "absent" }
"#,
    );
    fs::create_dir(repository.0.join("assets")).expect("excluded directory");
    repository.write("assets/policy.txt", "forbidden");
    let source = fs::read_to_string(repository.0.join("zrail.toml")).expect("contract");
    repository.write(
        "zrail.toml",
        &source.replace("[repository]\n", "[repository]\nexclude = ['assets/**']\n"),
    );
    violation(&repository, "raw", "REP-FILE-004");
    assert_eq!(coverage(&repository).repository_files[0].entries.len(), 1);
}

#[test]
fn exact_retired_paths_are_probed_inside_normally_pruned_directories() {
    let repository = configured(
        r#"
[[repository.files]]
name = "retired"
include = ["target/old-scanner"]
entry = "any"
reason = "Explicit absence cannot rely on cache pruning."
predicate = { kind = "exact-paths", paths = [] }
"#,
    );
    fs::create_dir(repository.0.join("target")).expect("cache directory");
    repository.write("target/old-scanner", "returned");
    violation(&repository, "retired", "REP-FILE-002");
}

#[test]
fn globs_fail_closed_when_unread_subtrees_may_contain_matches() {
    let repository = configured(
        r#"
[[repository.files]]
name = "all"
include = ["**/*.txt"]
reason = "Require a complete inventory."
predicate = { kind = "literal", text = "forbidden", mode = "absent" }
"#,
    );
    fs::create_dir(repository.0.join("target")).expect("unread directory");
    repository.write("target/bad.txt", "forbidden");
    incomplete(&repository);
    let source = fs::read_to_string(repository.0.join("zrail.toml")).expect("contract");
    repository.write(
        "zrail.toml",
        &source.replace(
            "include = [\"**/*.txt\"]",
            "include = ['**/*.txt']\nexclude = ['target/**']",
        ),
    );
    repository.lock();
    assert!(coverage(&repository).repository_files[0].satisfied);
}

#[test]
fn malformed_or_oversized_text_cannot_produce_partial_locks_or_coverage() {
    let repository = configured(
        r#"
[[repository.files]]
name = "text"
include = ["input"]
reason = "UTF-8 decoding and bounded reads are required for this claim."
predicate = { kind = "literal", text = "forbidden", mode = "absent" }
"#,
    );
    repository.write("input", "valid");
    repository.lock();
    let old_lock = fs::read(repository.0.join("zrail.lock")).expect("prior authority");
    fs::write(repository.0.join("input"), [0xff]).expect("malformed text");
    incomplete(&repository);
    fs::write(repository.0.join("input"), vec![b' '; 2 * 1024 * 1024 + 1])
        .expect("oversized input");
    incomplete(&repository);
    assert_eq!(
        fs::read(repository.0.join("zrail.lock")).expect("preserved authority"),
        old_lock
    );
}

#[cfg(unix)]
#[test]
fn file_links_are_contained_and_directory_link_globs_remain_unresolved() {
    use std::os::unix::fs::symlink;
    let repository = configured(
        r#"
[[repository.files]]
name = "linked"
include = ["assets/link.txt"]
reason = "An exact asset file link can be resolved and bound."
predicate = { kind = "literal", text = "allowed", mode = "equals" }
"#,
    );
    fs::create_dir(repository.0.join("assets")).expect("assets");
    repository.write("assets/value.txt", "allowed");
    symlink("value.txt", repository.0.join("assets/link.txt")).expect("contained file link");
    let report = coverage(&repository);
    assert!(report.repository_files[0].satisfied);
    assert_eq!(
        report.repository_files[0].entries[0]
            .resolved_path
            .as_deref(),
        Some("assets/value.txt")
    );
    fs::remove_file(repository.0.join("assets/link.txt")).expect("remove link");
    symlink(
        "/outside-does-not-exist",
        repository.0.join("assets/link.txt"),
    )
    .expect("broken outside link");
    incomplete(&repository);
    fs::remove_file(repository.0.join("assets/link.txt")).expect("remove broken link");
    symlink(".", repository.0.join("assets/link")).expect("directory link");
    let source = fs::read_to_string(repository.0.join("zrail.toml")).expect("contract");
    repository.write(
        "zrail.toml",
        &source.replace("assets/link.txt", "assets/**/*.txt"),
    );
    incomplete(&repository);
}

#[cfg(unix)]
#[test]
fn broken_links_count_as_present_without_reading_their_targets() {
    let repository = configured(
        r#"
[[repository.files]]
name = "retired"
include = ["retired"]
entry = "any"
reason = "A broken link still recreates a forbidden path."
predicate = { kind = "count", minimum = 0, maximum = 0 }
"#,
    );
    std::os::unix::fs::symlink("missing", repository.0.join("retired")).expect("broken link");
    violation(&repository, "retired", "REP-FILE-001");
    assert_eq!(
        coverage(&repository).repository_files[0].entries[0].kind,
        "symlink"
    );
}

fn incomplete(repository: &super::fixture::Repository) {
    let config = "zrail.toml".as_ref();
    for error in [
        zrail_rust::build_lock(&repository.0, config).expect_err("no partial lock"),
        zrail_rust::governed_surface_report(&repository.0, config)
            .expect_err("no partial coverage"),
        zrail_rust::check_repository(&repository.0, config, "zrail.lock".as_ref())
            .expect_err("no unenforced check"),
    ] {
        assert!(error.to_string().contains("REP-FILE-006"), "{error}");
    }
}
