//! Comment-preserving contract edits add only missing registered ratchets.

use std::{collections::BTreeSet, fmt::Write as _};
use zrail_core::RatchetContract;
use zrail_rust::BaselineRatchet;

#[derive(Debug)]
pub(super) struct BaselineEdit {
    pub(super) contract: String,
    pub(super) added: Vec<BaselineRatchet>,
    pub(super) preserved: Vec<BaselineRatchet>,
}

pub(super) fn merge(
    source: &str,
    existing: &[RatchetContract],
    candidates: Vec<BaselineRatchet>,
) -> BaselineEdit {
    let identities = existing
        .iter()
        .map(|ratchet| (ratchet.rule.as_str(), ratchet.target.as_str()))
        .collect::<BTreeSet<_>>();
    let (preserved, added): (Vec<_>, Vec<_>) = candidates
        .into_iter()
        .partition(|candidate| identities.contains(&(candidate.rule, candidate.target.as_str())));
    if added.is_empty() {
        return BaselineEdit {
            contract: source.to_owned(),
            added,
            preserved,
        };
    }
    let mut contract = source.to_owned();
    if !contract.ends_with('\n') {
        contract.push('\n');
    }
    for ratchet in &added {
        let _ = write!(
            contract,
            "\n[[ratchet]]\nrule = {}\ntarget = {}\nreason = {}\n",
            toml_string(ratchet.rule),
            toml_string(&ratchet.target),
            toml_string(ratchet.reason),
        );
    }
    BaselineEdit {
        contract,
        added,
        preserved,
    }
}

fn toml_string(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{0008}' => output.push_str("\\b"),
            '\t' => output.push_str("\\t"),
            '\n' => output.push_str("\\n"),
            '\u{000c}' => output.push_str("\\f"),
            '\r' => output.push_str("\\r"),
            control if control.is_control() && (control as u32) <= 0xffff => {
                let _ = write!(output, "\\u{:04x}", control as u32);
            }
            control if control.is_control() => {
                let _ = write!(output, "\\U{:08x}", control as u32);
            }
            other => output.push(other),
        }
    }
    output.push('"');
    output
}

#[cfg(test)]
#[path = "baseline_edit_test.rs"]
mod baseline_edit_test;
