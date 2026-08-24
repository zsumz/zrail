//! Exact report summaries and deterministic diagnostic groups.

use crate::diagnostic::{FindingTotals, Severity};

use super::{ReportGroup, ReportStatus, ReportSummary};

pub(super) fn summary_from_totals(totals: &FindingTotals, retained: usize) -> ReportSummary {
    let total = totals.total();
    ReportSummary {
        errors: totals.severity.get(&Severity::Error).copied().unwrap_or(0),
        warnings: totals
            .severity
            .get(&Severity::Warning)
            .copied()
            .unwrap_or(0),
        notes: totals.severity.get(&Severity::Note).copied().unwrap_or(0),
        retained,
        omitted: total.saturating_sub(retained),
    }
}

pub(super) fn groups_from_totals(totals: FindingTotals) -> Vec<ReportGroup> {
    totals
        .groups
        .into_iter()
        .map(|(group, count)| ReportGroup {
            id: group.id,
            rule: group.rule,
            severity: group.severity,
            count,
        })
        .collect()
}

pub(super) const fn status_from_summary(summary: ReportSummary) -> ReportStatus {
    if summary.errors == 0 {
        ReportStatus::Pass
    } else {
        ReportStatus::Fail
    }
}

pub(super) fn group_order(left: &ReportGroup, right: &ReportGroup) -> std::cmp::Ordering {
    (&left.id, &left.rule, left.severity).cmp(&(&right.id, &right.rule, right.severity))
}
