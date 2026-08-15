//! Human diff summaries account for every public change category.

use super::{ArchitectureChange, ChangeKind, DiffReport};

#[test]
fn human_summary_includes_neutral_changes() {
    let report = DiffReport::new(vec![ArchitectureChange::new(
        ChangeKind::Neutral,
        "repository",
        "root",
        "spelling changed without changing permission",
    )]);

    assert!(
        report
            .human()
            .contains("Changes: 0 grants, 0 revokes, 0 debt, 0 cleanup, 1 neutral, 0 unknown")
    );
}
