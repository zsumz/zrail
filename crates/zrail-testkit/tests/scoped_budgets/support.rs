//! Test-only source measurements and exact policy fragments.

pub(super) fn measured(lines: usize) -> String {
    format!(
        "//! Measured source.\npub struct State;\n{}",
        "// measured line\n".repeat(lines - 2)
    )
}

pub(super) fn size_finding<'a>(
    result: &'a zrail_rust::CheckResult,
    id: &str,
) -> Option<&'a zrail_core::Finding> {
    assert!(result.report.analysis.complete, "{:?}", result.report);
    result
        .report
        .findings
        .iter()
        .find(|finding| finding.id == id && finding.path.as_deref() == Some("src/child.rs"))
}

pub(super) const SCOPE: &str = r#"
[[source.rust.budgets.overrides]]
name = "core"
include = ["src/**"]
packages = ["fixture"]
roles = ["implementation"]
budget = { target = 5, soft = 8, hard = 10 }
reason = "Measured source family."
"#;

pub(super) const EXCEPTION: &str = r#"
[[source.rust.budgets.exceptions]]
path = "src/child.rs"
hard = 12
reason = "Split the existing seam."
owner = "architecture"
issue = "ARCH-42"
"#;

pub(super) const RATCHET: &str = r#"
[[ratchet]]
rule = "rust.file-size"
target = "src/child.rs"
baseline = 11
reason = "Exact reviewed measurement."
"#;
