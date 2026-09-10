//! Rename policies select source identifiers and retain alias identity, quantities and exact spans.

use super::support::{configured_subject, coverage, incomplete, violation};

const SUBJECT: &str = "kind='written-import-renames',names=['State']";
const SOURCE: &str = "//! Data.\npub struct State;\n";
const RENAME: &str = "use self::State as Alias;\n";

#[test]
fn required_rename_identity_rejects_alias_substitution_omission_and_quantity_changes() {
    let source = format!("{SOURCE}{RENAME}");
    let repository = configured_subject(
        SUBJECT,
        "kind='exact-counts',counts=[{path='src/child.rs',name='State as Alias',count=1}]",
        &source,
    );
    repository.lock();
    assert_eq!(
        repository.check().report.status,
        zrail_core::ReportStatus::Pass
    );
    let before = coverage(&repository);
    let inventory = &before.rust_inventories[0];
    assert_eq!(inventory.claim, "authored-rust-import-rename-syntax");
    assert_eq!(inventory.counts[0].name, "State as Alias");
    let span = inventory.occurrence_sample[0].span;
    let line = source.lines().nth(span.line - 1).expect("source line");
    assert_eq!(
        &line[span.column - 1..span.end_column - 1],
        "State as Alias"
    );
    assert_eq!(
        repository.explain("src/child.rs").rust_inventories,
        before.rust_inventories
    );
    for replacement in [
        "",
        "use self::State as Other;",
        "use self::State as _;",
        "fn plain() { use self::State; }",
        RENAME,
    ] {
        let source = if replacement == RENAME {
            format!("{source}fn nested() {{ {RENAME} }}\n")
        } else {
            format!("{SOURCE}{replacement}")
        };
        repository.write("src/child.rs", &source);
        violation(&repository);
        assert!(
            repository
                .check()
                .report
                .findings
                .iter()
                .any(|finding| finding.id == "RUST-INVENTORY-001"
                    && finding.message.contains("authored import-rename"))
        );
    }
}

#[test]
fn rename_prohibitions_remain_active_without_inheriting_alias_destination_authority() {
    let repository = configured_subject(SUBJECT, "kind='count',maximum=0", SOURCE);
    assert!(coverage(&repository).rust_inventories[0].satisfied);
    repository.write(
        "src/child.rs",
        "//! Data.\npub struct Other;\npub use self::Other as State;\n",
    );
    assert!(coverage(&repository).rust_inventories[0].satisfied);
    repository.write("src/child.rs", &format!("{SOURCE}{RENAME}"));
    violation(&repository);
    repository.write("src/child.rs", "use broken::{");
    incomplete(&repository);
}

#[test]
fn renamed_owner_sets_deduplicate_members_but_keep_every_physical_occurrence() {
    let repository = configured_subject(
        SUBJECT,
        "kind='exact-owners',owners=[{path='src/shared.rs',name='State as Alias'}]",
        "//! Mounts.\npub struct State;\nmod a { include!(\"shared.rs\"); } mod b { include!(\"shared.rs\"); }\n",
    );
    repository.write("src/shared.rs", "//! Renames.\n#[cfg(test)] use super::State as Alias;\n#[cfg(not(test))] use super::State as Alias;\n");
    let report = coverage(&repository);
    assert!(report.rust_inventories[0].satisfied);
    assert_eq!(report.rust_inventories[0].observed_count, 2);
    assert_eq!(report.rust_inventories[0].counts.len(), 1);
    assert_eq!(
        report.json().expect("JSON"),
        coverage(&repository).json().expect("repeat JSON")
    );
}
