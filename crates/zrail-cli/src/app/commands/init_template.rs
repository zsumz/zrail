//! Conservative Rust starter contracts rendered from discovered Cargo package roots.

use std::fmt::Write as _;

pub(super) fn render(roots: &[String]) -> String {
    let roots = roots
        .iter()
        .map(|root| toml_string(root))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
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
tests = "sibling"

[source.rust.hygiene]
unsafe = "allow"
lint_suppressions = "allow"
deny_methods = []
deny_macros = []

[source.rust.size.facade]
target = 300
hard = 300

[source.rust.size.implementation]
target = 300
hard = 300

[source.rust.size.test]
target = 300
hard = 300

[source.rust.size.auxiliary]
target = 300
hard = 300
"#
    )
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
