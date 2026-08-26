//! Stable mirror-plan parsing and rendering.

use super::{MirrorPlan, MirrorVerification};

impl MirrorPlan {
    /// Serializes this plan as deterministic pretty JSON with a trailing newline.
    pub fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self).map(|mut output| {
            output.push('\n');
            output
        })
    }

    /// Parses strict plan JSON and verifies its embedded canonical digest.
    pub fn parse(source: &str) -> Result<Self, String> {
        let plan = serde_json::from_str::<Self>(source)
            .map_err(|error| format!("invalid mirror plan JSON: {error}"))?;
        if plan.schema != 1 {
            return Err(format!(
                "unsupported mirror plan schema {}; expected 1",
                plan.schema
            ));
        }
        let expected = plan.expected_sha256()?;
        if plan.plan_sha256 != expected {
            return Err("mirror plan digest does not match its canonical payload".into());
        }
        if !plan
            .mirrors
            .windows(2)
            .all(|pair| pair[0].policy_id < pair[1].policy_id)
        {
            return Err("mirror plan entries must be unique and canonically sorted".into());
        }
        Ok(plan)
    }

    /// Renders a compact human summary without omitting plan authority.
    pub fn human(&self) -> String {
        format!(
            "Mirror plan sha256:{}\n{} exact mirror(s) across {} execution group(s)\n",
            self.plan_sha256,
            self.mirrors.len(),
            self.mirrors
                .iter()
                .map(|mirror| mirror.execution_group.as_str())
                .collect::<std::collections::BTreeSet<_>>()
                .len()
        )
    }
}

impl MirrorVerification {
    /// Serializes this verification as deterministic pretty JSON.
    pub fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self).map(|mut output| {
            output.push('\n');
            output
        })
    }

    /// Renders exact plan identity followed by mirror diagnostics.
    pub fn human(&self) -> String {
        format!(
            "Verified mirror plan sha256:{}\n{} exact mirror(s)\n{}",
            self.plan_sha256,
            self.mirrors,
            self.report.human()
        )
    }
}
