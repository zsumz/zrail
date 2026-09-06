//! Selection gaps fail closed; absence and display limits never fabricate a positive match.

use super::support::{SOURCE, configured, coverage, exact, incomplete, violation};

#[test]
fn zero_and_empty_exact_maps_remain_enforced_when_subjects_or_files_are_absent() {
    for assertion in ["kind='count',maximum=0", "kind='exact-counts',counts=[]"] {
        let repository = configured(assertion, "//! Data.\npub struct State;\n");
        repository.lock();
        assert!(coverage(&repository).rust_inventories[0].satisfied);
        repository.write("src/child.rs", SOURCE);
        violation(&repository);
        let policy = std::fs::read_to_string(repository.0.join("zrail.toml")).expect("contract");
        repository.write(
            "zrail.toml",
            &policy.replace("src/**/*.rs", "src/absent.rs"),
        );
        assert!(coverage(&repository).rust_inventories[0].satisfied);
    }
    let repository = configured("kind='count',minimum=1", SOURCE);
    let policy = std::fs::read_to_string(repository.0.join("zrail.toml")).expect("contract");
    repository.write(
        "zrail.toml",
        &policy.replace("src/**/*.rs", "src/absent.rs"),
    );
    violation(&repository);
}

#[test]
fn source_exclusions_and_malformed_or_expression_only_files_cannot_hide_selected_code() {
    let repository = configured(&exact("src/child.rs", "poll", 1), SOURCE);
    let original = std::fs::read_to_string(repository.0.join("zrail.toml")).expect("contract");
    repository.write(
        "zrail.toml",
        &original.replace("[repository]", "[repository]\nexclude=['src/child.rs']"),
    );
    incomplete(&repository);
    repository.write("zrail.toml", &original);
    for source in ["fn broken( {", "{ self.poll(); }"] {
        repository.write("src/child.rs", source);
        incomplete(&repository);
    }
}

#[test]
fn exact_inventory_requires_the_selected_file_after_its_mount_is_removed() {
    let repository = configured(&exact("src/child.rs", "poll", 1), SOURCE);
    std::fs::remove_file(repository.0.join("src/child.rs")).expect("delete reviewed file");
    repository.write("src/lib.rs", "//! Empty.\n");
    violation(&repository);
}

#[test]
fn complete_quantities_are_independent_of_location_sample_truncation() {
    let source = SOURCE.replace("self.poll();", &"self.poll();".repeat(40));
    let repository = configured(&exact("src/child.rs", "poll", 40), &source);
    let observed = coverage(&repository).rust_inventories.remove(0);
    assert!(observed.satisfied);
    assert_eq!(observed.observed_count, 40);
    assert_eq!(observed.counts[0].count, 40);
    assert_eq!(observed.occurrence_sample.len(), 16);
    assert_eq!(observed.occurrences_omitted, 24);
    repository.write(
        "src/child.rs",
        &SOURCE.replace("self.poll();", &"self.poll();".repeat(41)),
    );
    violation(&repository);
}

#[cfg(unix)]
#[test]
fn symlink_and_undeclared_workspace_boundaries_cannot_produce_trusted_inventories() {
    let repository = configured(&exact("src/child.rs", "poll", 1), SOURCE);
    std::os::unix::fs::symlink("child.rs", repository.0.join("src/alias.rs"))
        .expect("fixture link");
    incomplete(&repository);
    std::fs::remove_file(repository.0.join("src/alias.rs")).expect("remove fixture link");
    std::fs::create_dir(repository.0.join("src/nested")).expect("nested source");
    repository.write(
        "src/nested/Cargo.toml",
        "[package]\nname='nested'\nversion='0.0.0'\n[workspace]\n",
    );
    repository.write("src/nested/lib.rs", SOURCE);
    incomplete(&repository);
}
