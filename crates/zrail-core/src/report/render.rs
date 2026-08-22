//! Human report rendering.

use std::fmt::Write as _;

use crate::{AnalysisQuality, Severity};

use super::{Report, ReportStatus};

pub(super) fn human(report: &Report) -> String {
    let mut output = String::new();
    for finding in &report.findings {
        let _ = writeln!(
            output,
            "{}[{}]: {}",
            severity_name(finding.severity),
            finding.id,
            finding.message
        );
        if let Some(path) = &finding.path {
            if let Some(span) = finding.span {
                let _ = writeln!(output, "  --> {path}:{}:{}", span.line, span.column);
            } else {
                let _ = writeln!(output, "  --> {path}");
            }
        }
        let _ = writeln!(output, "   = rule: {}", finding.rule);
        let _ = writeln!(output, "   = analysis: {}", analysis_name(finding.analysis));
        if let Some(reason) = &finding.reason {
            let _ = writeln!(output, "   = reason: {reason}");
        }
        if let Some(help) = &finding.help {
            let _ = writeln!(output, "   = help: {help}");
        }
        output.push('\n');
    }
    let _ = writeln!(
        output,
        "Diagnostics: {} total; showing {}.",
        grouped_number(report.summary.total()),
        grouped_number(report.summary.retained)
    );
    output.push('\n');
    render_groups(&mut output, report);
    output.push('\n');
    let _ = writeln!(
        output,
        "Status: {} ({} errors, {} warnings, {} notes)",
        status_name(report.status),
        grouped_number(report.summary.errors),
        grouped_number(report.summary.warnings),
        grouped_number(report.summary.notes)
    );
    output
}

fn render_groups(output: &mut String, report: &Report) {
    output.push_str("By rule:\n");
    if report.groups.is_empty() {
        output.push_str("  none\n");
        return;
    }
    let id_width = report
        .groups
        .iter()
        .map(|group| group.id.len())
        .max()
        .unwrap_or(0);
    let rule_width = report
        .groups
        .iter()
        .map(|group| group.rule.len())
        .max()
        .unwrap_or(0);
    for group in &report.groups {
        let _ = writeln!(
            output,
            "  {:id_width$}  {:rule_width$}  {:7}  {}",
            group.id,
            group.rule,
            severity_name(group.severity),
            grouped_number(group.count)
        );
    }
}

fn grouped_number(value: usize) -> String {
    let digits = value.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.bytes().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(char::from(digit));
    }
    grouped
}

const fn analysis_name(quality: AnalysisQuality) -> &'static str {
    match quality {
        AnalysisQuality::Exact => "exact",
        AnalysisQuality::Conservative => "conservative",
        AnalysisQuality::Unresolved => "unresolved",
    }
}

const fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Note => "note",
    }
}

const fn status_name(status: ReportStatus) -> &'static str {
    match status {
        ReportStatus::Pass => "pass",
        ReportStatus::Fail => "fail",
        ReportStatus::Invalid => "invalid",
    }
}
