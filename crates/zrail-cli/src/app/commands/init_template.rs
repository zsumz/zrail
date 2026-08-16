//! Conservative Rust starter contracts rendered from discovered Cargo package roots.

use std::fmt::Write as _;

use zrail_rust::BaselinePlan;

use crate::app::args::InitPreset;

pub(super) fn render(roots: &[String], preset: InitPreset, baseline: &BaselinePlan) -> String {
    let roots = roots
        .iter()
        .map(|root| toml_string(root))
        .collect::<Vec<_>>()
        .join(", ");
    let mut contract = format!(
        r#"schema = 1
adapters = ["rust"]

[repository]
roots = [{roots}]
exclude = []
workspace_members = "exact"
nested_git = "deny"
submodules = "deny"
symlinks = "inside"

[dependencies]
mode = "locked"
unassigned_packages = "allow"
cycles = "deny"

[source.rust]
module_docs = "allow"
facades = "allow"
entrypoints = "allow"
tests = "{tests}"

[source.rust.macros]
mode = "{macros}"

[source.rust.hygiene]
unsafe = "allow"
lint_suppressions = "allow"
deny_methods = []
deny_macros = []
"#,
        tests = match preset {
            InitPreset::Zsumz => "sibling",
            InitPreset::Rust => "allow",
        },
        macros = match preset {
            InitPreset::Zsumz => "deny-unreviewed",
            InitPreset::Rust => "allow",
        },
    );
    if preset == InitPreset::Zsumz {
        write_size(&mut contract, baseline);
    }
    for ratchet in &baseline.ratchets {
        let _ = write!(
            contract,
            "\n[[ratchet]]\nrule = {}\ntarget = {}\nreason = {}\n",
            toml_string(ratchet.rule),
            toml_string(&ratchet.target),
            toml_string(ratchet.reason),
        );
    }
    contract
}

fn write_size(contract: &mut String, baseline: &BaselinePlan) {
    let hard =
        |select: fn(&zrail_rust::BaselineSize) -> usize| baseline.size.as_ref().map_or(300, select);
    let _ = write!(
        contract,
        r"
[source.rust.size.facade]
target = 300
hard = {facade}

[source.rust.size.implementation]
target = 300
hard = {implementation}

[source.rust.size.test]
target = 300
hard = {test}

[source.rust.size.auxiliary]
target = 300
hard = {auxiliary}
",
        facade = hard(|size| size.facade_hard),
        implementation = hard(|size| size.implementation_hard),
        test = hard(|size| size.test_hard),
        auxiliary = hard(|size| size.auxiliary_hard),
    );
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
#[path = "init_template_test.rs"]
mod init_template_test;
