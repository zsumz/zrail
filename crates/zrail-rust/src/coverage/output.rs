//! Human and JSON renderers preserve the deterministic report model.

use std::fmt::Write as _;

use super::GovernedSurfaceReport;

impl GovernedSurfaceReport {
    /// Serializes the complete report as pretty schema-versioned JSON.
    pub fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self).map(|mut output| {
            output.push('\n');
            output
        })
    }

    /// Renders a concise human audit summary with canonical policy identities.
    pub fn human(&self) -> String {
        let metrics = self.analysis.metrics;
        let mut output = format!(
            "Governed surface schema {} (contract schema {}, {}, complete)\n",
            self.schema, self.contract_schema, self.contract_sha256
        );
        let _ = writeln!(
            output,
            "Analysis: {} Rust files, {} facts, {} base contexts, {} derived contexts",
            metrics.physical_rust_files,
            metrics.physical_facts,
            metrics.base_contexts,
            metrics.derived_contexts
        );
        let exclusions = if self.analysis.exclusions.is_empty() {
            "none".into()
        } else {
            self.analysis.exclusions.join(", ")
        };
        let _ = writeln!(output, "Exclusions: {exclusions}");
        let _ = writeln!(output, "Enabled rails: {}", self.enabled_rails.len());
        let _ = writeln!(
            output,
            "Owners: {} ({} unresolved, {} ambiguous occurrences)",
            self.owners.len(),
            self.unresolved_occurrences,
            self.ambiguous_occurrences
        );
        for owner in &self.owners {
            let _ = writeln!(
                output,
                "  {} -> {} ({} occurrences)",
                owner.policy_id,
                owner.target,
                owner.occurrences.len()
            );
        }
        let _ = writeln!(
            output,
            "Dependency prohibitions: {}",
            self.dependencies.len()
        );
        for dependency in &self.dependencies {
            let _ = writeln!(
                output,
                "  {} from {} ({} prohibited paths)",
                dependency.policy_id,
                dependency.from,
                dependency.paths.len()
            );
        }
        let _ = writeln!(output, "Test mirrors: {}", self.test_mirrors.len());
        output
    }
}
