//! Public schema for deterministic mirror execution plans.

use serde::{Deserialize, Serialize};
use zrail_core::{Report, TestExecutionIdentity, TestMirrorContract, sha256_hex};

use crate::AnalysisMetrics;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Exact non-executing plan consumed by a separately trusted receipt producer.
pub struct MirrorPlan {
    /// Mirror-plan schema; currently `1`.
    pub schema: u64,
    /// SHA-256 of the exact resolved contract bundle.
    pub contract_sha256: String,
    /// Deterministic analysis work census for the planned repository.
    pub analysis: AnalysisMetrics,
    /// SHA-256 of the canonical plan payload excluding this field.
    pub plan_sha256: String,
    /// Every exact mirror in canonical policy order.
    pub mirrors: Vec<PlannedTestMirror>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One exact test mirror and expected content-bound receipt identity.
pub struct PlannedTestMirror {
    /// Stable mirror policy identity.
    pub policy_id: String,
    /// Exact production source path.
    pub production: String,
    /// Exact Cargo-test source path.
    pub test: String,
    /// Exact named test declaration.
    pub test_name: String,
    /// Exact output path for the schema-2 receipt.
    pub receipt: String,
    /// Additional exact reviewed input paths.
    pub inputs: Vec<String>,
    /// Expected digest of source, inputs, test identity, and execution identity.
    pub input_sha256: String,
    /// Exact command, package, features, target, and toolchain identity.
    pub execution: TestExecutionIdentity,
    /// Stable digest grouping mirrors with identical execution identity.
    pub execution_group: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
/// Receipt verification result for one current, digest-bound mirror plan.
pub struct MirrorVerification {
    /// Verification schema; currently `1`.
    pub schema: u64,
    /// Digest of the exact plan that was verified.
    pub plan_sha256: String,
    /// Number of exact mirror receipts required by the plan.
    pub mirrors: usize,
    /// Mirror and receipt findings; pass means every exact receipt is authoritative.
    pub report: Report,
}

impl MirrorPlan {
    pub(super) fn new(
        contract_sha256: String,
        analysis: AnalysisMetrics,
        mirrors: Vec<PlannedTestMirror>,
    ) -> Result<Self, String> {
        let mut plan = Self {
            schema: 1,
            contract_sha256,
            analysis,
            plan_sha256: String::new(),
            mirrors,
        };
        plan.plan_sha256 = plan.expected_sha256()?;
        Ok(plan)
    }

    pub(super) fn expected_sha256(&self) -> Result<String, String> {
        let payload = (
            self.schema,
            self.contract_sha256.as_str(),
            self.analysis,
            self.mirrors.as_slice(),
        );
        serde_json::to_vec(&payload)
            .map(|bytes| sha256_hex(&bytes))
            .map_err(|error| format!("serialize mirror plan payload: {error}"))
    }
}

impl PlannedTestMirror {
    pub(super) fn new(mirror: &TestMirrorContract, input_sha256: String) -> Result<Self, String> {
        let execution = serde_json::to_vec(&mirror.execution)
            .map_err(|error| format!("serialize mirror execution identity: {error}"))?;
        Ok(Self {
            policy_id: policy_id(mirror),
            production: mirror.production.clone(),
            test: mirror.test.clone(),
            test_name: mirror.name.clone(),
            receipt: mirror.receipt.clone(),
            inputs: mirror.inputs.clone(),
            input_sha256,
            execution: mirror.execution.clone(),
            execution_group: sha256_hex(&execution),
        })
    }
}

pub(crate) fn policy_id(mirror: &TestMirrorContract) -> String {
    policy_id_fields(&mirror.production, &mirror.test, &mirror.name)
}

fn policy_id_fields(production: &str, test: &str, name: &str) -> String {
    let mut framed = b"zrail-test-mirror-policy-v1\0".to_vec();
    for value in [production, test, name] {
        framed.extend_from_slice(value.len().to_string().as_bytes());
        framed.push(0);
        framed.extend_from_slice(value.as_bytes());
        framed.push(0xff);
    }
    format!("test-mirror:sha256:{}", sha256_hex(&framed))
}

#[cfg(test)]
#[path = "model_test.rs"]
mod model_test;
