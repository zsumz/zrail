//! Exact ownership preserves locations while retaining independent complete occurrence counts.

use super::support::{SOURCE, configured, coverage, violation};

const OWNER: &str = "kind='exact-owners',owners=[{path='src/child.rs',name='poll'}]";

#[test]
fn required_owners_accept_duplicates_but_reject_omissions_and_substitutions() {
    let repository = configured(OWNER, SOURCE);
    repository.lock();
    assert_eq!(
        repository.check().report.status,
        zrail_core::ReportStatus::Pass
    );
    let first = coverage(&repository);
    assert_eq!(first.rust_inventories[0].counts[0].count, 1);
    repository.write(
        "src/child.rs",
        &SOURCE.replace("self.poll();", &"self.poll();".repeat(32)),
    );
    let repeated = coverage(&repository);
    let inventory = &repeated.rust_inventories[0];
    assert!(inventory.satisfied);
    assert_eq!(inventory.observed_count, 32);
    assert_eq!(inventory.counts[0].count, 32);
    assert_eq!(inventory.occurrence_sample.len(), 16);
    assert_eq!(inventory.occurrences_omitted, 16);
    assert_ne!(inventory.inputs, first.rust_inventories[0].inputs);
    assert_eq!(
        repository.explain("src/child.rs").rust_inventories,
        repeated.rust_inventories
    );
    for replacement in [
        "",
        "self.wake();",
        "Self::poll(self);",
        "let _f = Self::poll;",
    ] {
        repository.write("src/child.rs", &SOURCE.replace("self.poll();", replacement));
        violation(&repository);
    }
}

#[test]
fn complete_ownership_sets_reject_moved_and_additional_locations() {
    let repository = configured(OWNER, SOURCE);
    repository.write("src/moved.rs", SOURCE);
    repository.write(
        "src/lib.rs",
        "//! Wiring.\nmod child; mod moved;\npub use child::State;\n",
    );
    violation(&repository);
    assert_eq!(coverage(&repository).rust_inventories[0].counts.len(), 2);
    repository.write("src/child.rs", &SOURCE.replace("self.poll();", ""));
    violation(&repository);
    assert_eq!(coverage(&repository).rust_inventories[0].observed_count, 1);
    let diagnostic = repository
        .check()
        .report
        .findings
        .into_iter()
        .find(|finding| finding.id == "RUST-INVENTORY-001")
        .expect("owner diagnostic");
    assert!(diagnostic.message.contains("exact 1 file/subject owners"));
}

#[test]
fn empty_owner_sets_stay_active_and_required_owners_fail_when_files_disappear() {
    let repository = configured(
        "kind='exact-owners',owners=[]",
        &SOURCE.replace("self.poll();", ""),
    );
    assert!(coverage(&repository).rust_inventories[0].satisfied);
    repository.write("src/child.rs", SOURCE);
    violation(&repository);
    drop(repository);
    let repository = configured(OWNER, SOURCE);
    repository.write("src/lib.rs", "//! Empty.\n");
    std::fs::remove_file(repository.0.join("src/child.rs")).expect("remove source");
    violation(&repository);
    assert_eq!(coverage(&repository).rust_inventories[0].observed_count, 0);
}
