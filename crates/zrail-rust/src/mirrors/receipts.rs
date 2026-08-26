//! Plan-bound bulk result input and deterministic schema-2 receipt rendering.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use zrail_core::{
    EXECUTION_RECEIPT_SCHEMA, ExecutionReceipt, ExecutionReceiptStatus, ExecutionReceiptTest,
    sha256_hex, validate_execution_receipt, versioned_producer,
};

use super::MirrorPlan;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Exact outcomes emitted by a separately trusted mirror-test producer.
pub struct MirrorResultSet {
    /// Result schema; currently `1`.
    pub schema: u64,
    /// Digest of the exact mirror plan executed by the producer.
    pub plan_sha256: String,
    /// Versioned producer identity formatted as `name major.minor.patch`.
    pub producer: String,
    /// Outcomes grouped by exact execution identity.
    pub groups: Vec<MirrorExecutionResult>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Outcomes for one exact execution group in a mirror plan.
pub struct MirrorExecutionResult {
    /// Stable digest of the plan's command, package, features, target, and toolchain.
    pub execution_group: String,
    /// Exact policy outcomes in canonical policy order.
    pub tests: Vec<MirrorTestResult>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One plan policy outcome reported by the trusted producer.
pub struct MirrorTestResult {
    /// Stable policy identity from the mirror plan.
    pub policy_id: String,
    /// Observed execution outcome.
    pub status: ExecutionReceiptStatus,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
/// Canonical collection of receipt files rendered from one current plan.
pub struct MirrorReceiptBundle {
    /// Bundle schema; currently `1`.
    pub schema: u64,
    /// Digest of the exact plan bound by every receipt.
    pub plan_sha256: String,
    /// Versioned identity copied into every rendered receipt.
    pub producer: String,
    /// Receipt artifacts in canonical output-path order.
    pub receipts: Vec<RenderedMirrorReceipt>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
/// One exact repository-relative receipt artifact.
pub struct RenderedMirrorReceipt {
    /// Stable mirror policy identity.
    pub policy_id: String,
    /// Exact repository-relative output path declared by policy.
    pub path: String,
    /// SHA-256 of the canonical receipt JSON bytes, including its trailing newline.
    pub sha256: String,
    /// Exact strict schema-2 JSON bytes to write at `path`.
    pub source: String,
}

impl MirrorResultSet {
    /// Parses strict result JSON and rejects non-canonical or stale identities.
    pub fn parse(source: &str) -> Result<Self, String> {
        let results = serde_json::from_str::<Self>(source)
            .map_err(|error| format!("invalid mirror result JSON: {error}"))?;
        if results.schema != 1 {
            return Err(format!(
                "unsupported mirror result schema {}; expected 1",
                results.schema
            ));
        }
        if !versioned_producer(&results.producer) {
            return Err("mirror result producer must be `name major.minor.patch`".into());
        }
        if !results
            .groups
            .windows(2)
            .all(|pair| pair[0].execution_group < pair[1].execution_group)
        {
            return Err("mirror result groups must be unique and canonically sorted".into());
        }
        if results.groups.iter().any(|group| {
            group
                .tests
                .windows(2)
                .any(|pair| pair[0].policy_id >= pair[1].policy_id)
        }) {
            return Err("mirror result tests must be unique and canonically sorted".into());
        }
        Ok(results)
    }
}

impl MirrorReceiptBundle {
    pub(super) fn render(plan: &MirrorPlan, results: MirrorResultSet) -> Result<Self, String> {
        if results.plan_sha256 != plan.plan_sha256 {
            return Err("mirror results do not bind the current plan digest".into());
        }
        let expected_groups = plan
            .mirrors
            .iter()
            .map(|mirror| mirror.execution_group.as_str())
            .collect::<BTreeSet<_>>();
        let observed_groups = results
            .groups
            .iter()
            .map(|group| group.execution_group.as_str())
            .collect::<BTreeSet<_>>();
        if expected_groups != observed_groups {
            return Err("mirror results do not report every exact execution group once".into());
        }
        let mut outcomes = BTreeMap::new();
        for group in &results.groups {
            for test in &group.tests {
                let Some(mirror) = plan
                    .mirrors
                    .iter()
                    .find(|mirror| mirror.policy_id == test.policy_id)
                else {
                    return Err(format!(
                        "mirror results contain unknown policy {:?}",
                        test.policy_id
                    ));
                };
                if mirror.execution_group != group.execution_group {
                    return Err(format!(
                        "mirror result {:?} is assigned to the wrong execution group",
                        test.policy_id
                    ));
                }
                if outcomes
                    .insert(test.policy_id.as_str(), test.status)
                    .is_some()
                {
                    return Err(format!(
                        "mirror results contain duplicate policy {:?}",
                        test.policy_id
                    ));
                }
            }
        }
        if outcomes.len() != plan.mirrors.len() {
            return Err("mirror results do not report every exact planned policy".into());
        }
        let mut paths = BTreeSet::new();
        let mut receipts = Vec::with_capacity(plan.mirrors.len());
        for mirror in &plan.mirrors {
            if !paths.insert(mirror.receipt.as_str()) {
                return Err(format!(
                    "mirror plan contains duplicate receipt path {:?}",
                    mirror.receipt
                ));
            }
            let status = outcomes
                .get(mirror.policy_id.as_str())
                .copied()
                .ok_or_else(|| {
                    format!("missing result for mirror policy {:?}", mirror.policy_id)
                })?;
            let receipt = ExecutionReceipt {
                schema: EXECUTION_RECEIPT_SCHEMA,
                producer: results.producer.clone(),
                input_sha256: mirror.input_sha256.clone(),
                execution: mirror.execution.clone(),
                tests: vec![ExecutionReceiptTest {
                    id: mirror.test_name.clone(),
                    status,
                }],
            };
            validate_execution_receipt(&receipt)?;
            let bytes = canonical_receipt(&receipt)
                .map_err(|error| format!("serialize execution receipt: {error}"))?;
            receipts.push(RenderedMirrorReceipt {
                policy_id: mirror.policy_id.clone(),
                path: mirror.receipt.clone(),
                sha256: sha256_hex(bytes.as_bytes()),
                source: bytes,
            });
        }
        receipts.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(Self {
            schema: 1,
            plan_sha256: plan.plan_sha256.clone(),
            producer: results.producer,
            receipts,
        })
    }

    /// Serializes the complete bundle as deterministic pretty JSON.
    pub fn json(&self) -> Result<String, serde_json::Error> {
        pretty_json(self)
    }

    /// Renders a compact summary without omitting plan or producer identity.
    pub fn human(&self) -> String {
        format!(
            "Rendered {} schema-2 receipt(s) for mirror plan sha256:{}\nProducer: {}\n",
            self.receipts.len(),
            self.plan_sha256,
            self.producer
        )
    }
}

fn canonical_receipt(receipt: &ExecutionReceipt) -> Result<String, serde_json::Error> {
    pretty_json(receipt)
}

fn pretty_json(value: &impl Serialize) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(value).map(|mut output| {
        output.push('\n');
        output
    })
}

#[cfg(test)]
#[path = "receipts_test.rs"]
mod receipts_test;
