//! Baseline output exposes added, preserved, and rejected adoption candidates.

use std::fmt::Write as _;

use zrail_core::{DiffReport, Finding, Severity};
use zrail_rust::BaselineRatchet;

use crate::app::output::{OutputFormat, json_escape};

use super::baseline_plan::PreparedBaseline;

#[derive(Clone, Copy)]
pub(super) enum BaselineStatus {
    DryRun,
    Refused,
    Rejected,
    Updated,
}

pub(super) fn render(
    plan: &PreparedBaseline,
    authority: Option<&DiffReport>,
    status: BaselineStatus,
    format: OutputFormat,
) -> String {
    match format {
        OutputFormat::Human => human(plan, authority, status),
        OutputFormat::Json => json(plan, authority, status),
    }
}

fn human(
    plan: &PreparedBaseline,
    authority: Option<&DiffReport>,
    status: BaselineStatus,
) -> String {
    let heading = match status {
        BaselineStatus::DryRun => "zrail baseline dry run",
        BaselineStatus::Refused => "zrail baseline refused architecture grants",
        BaselineStatus::Rejected => "zrail baseline refused non-ratchetable violations",
        BaselineStatus::Updated => "zrail baseline updated architecture state",
    };
    let mut output = format!(
        "{heading}\n\nroot: {}\nadded: {}\npreserved: {}\nrejected: {}\n",
        plan.root.display(),
        plan.added.len(),
        plan.preserved.len(),
        rejected(plan).count()
    );
    render_ratchets(&mut output, "added", &plan.added);
    render_ratchets(&mut output, "preserved", &plan.preserved);
    if matches!(status, BaselineStatus::Rejected) {
        output.push('\n');
        output.push_str(&plan.report.human());
    }
    if let Some(authority) = authority {
        output.push_str("\nAuthority changes:\n");
        output.push_str(&authority.human());
    }
    match status {
        BaselineStatus::DryRun => output.push_str("\nNo files written.\n"),
        BaselineStatus::Refused => {
            output.push_str("\nRerun with `--accept-grants` after human review.\n");
        }
        BaselineStatus::Rejected | BaselineStatus::Updated => {}
    }
    output
}

fn render_ratchets(output: &mut String, label: &str, ratchets: &[BaselineRatchet]) {
    for ratchet in ratchets {
        let selector = ratchet
            .selector
            .as_ref()
            .map_or_else(String::new, |selector| format!("[{selector}]"));
        let _ = writeln!(
            output,
            "{label}: {}{selector} {}",
            ratchet.rule, ratchet.target
        );
    }
}

fn json(plan: &PreparedBaseline, authority: Option<&DiffReport>, status: BaselineStatus) -> String {
    let mut output = format!(
        concat!(
            "{{\n",
            "  \"schema\": 1,\n",
            "  \"status\": \"{}\",\n",
            "  \"root\": \"{}\",\n",
            "  \"added\": {},\n",
            "  \"preserved\": {},\n",
            "  \"rejected\": {},\n",
            "  \"authority\": {}\n",
            "}}\n"
        ),
        status_name(status),
        json_escape(&plan.root.to_string_lossy()),
        ratchets_json(&plan.added),
        ratchets_json(&plan.preserved),
        findings_json(rejected(plan)),
        authority_json(authority),
    );
    output.shrink_to_fit();
    output
}

fn ratchets_json(ratchets: &[BaselineRatchet]) -> String {
    let values = ratchets
        .iter()
        .map(|ratchet| {
            let selector = ratchet.selector.as_deref().map_or_else(
                || "null".into(),
                |selector| format!("\"{}\"", json_escape(selector)),
            );
            format!(
                "{{\"rule\":\"{}\",\"selector\":{},\"target\":\"{}\",\"reason\":\"{}\"}}",
                json_escape(ratchet.rule),
                selector,
                json_escape(&ratchet.target),
                json_escape(ratchet.reason)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
}

fn findings_json<'a>(findings: impl Iterator<Item = &'a Finding>) -> String {
    let values = findings
        .map(|finding| {
            format!(
                "{{\"id\":\"{}\",\"rule\":\"{}\",\"path\":{},\"message\":\"{}\"}}",
                json_escape(&finding.id),
                json_escape(&finding.rule),
                finding.path.as_deref().map_or_else(
                    || "null".into(),
                    |path| format!("\"{}\"", json_escape(path))
                ),
                json_escape(&finding.message)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
}

fn rejected(plan: &PreparedBaseline) -> impl Iterator<Item = &Finding> {
    plan.report
        .findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
}

fn authority_json(authority: Option<&DiffReport>) -> String {
    authority.map_or_else(
        || "null".into(),
        |report| {
            format!(
                "{{\"grants\":{},\"debt\":{},\"unknown\":{}}}",
                report.summary.grants, report.summary.debt, report.summary.unknown
            )
        },
    )
}

const fn status_name(status: BaselineStatus) -> &'static str {
    match status {
        BaselineStatus::DryRun => "dry-run",
        BaselineStatus::Refused => "refused",
        BaselineStatus::Rejected => "rejected",
        BaselineStatus::Updated => "updated",
    }
}
