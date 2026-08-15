//! Shared output format and error rendering.

use std::fmt::Write as _;

use super::error::CliError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OutputFormat {
    Human,
    Json,
}

pub(crate) fn render_error(error: &CliError, format: OutputFormat) -> String {
    match format {
        OutputFormat::Human => {
            let mut output = format!("error: {}\n", error.message);
            if let Some(help) = &error.help {
                let _ = writeln!(output, "help: {help}");
            }
            output
        }
        OutputFormat::Json => {
            let message = json_escape(&error.message);
            let help = error.help.as_deref().map_or_else(
                || "null".into(),
                |value| format!("\"{}\"", json_escape(value)),
            );
            format!(
                concat!(
                    "{{\n",
                    "  \"schema\": 1,\n",
                    "  \"status\": \"invalid\",\n",
                    "  \"error\": \"{message}\",\n",
                    "  \"help\": {help}\n",
                    "}}\n",
                ),
                message = message,
                help = help,
            )
        }
    }
}

pub(crate) fn json_escape(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{0008}' => output.push_str("\\b"),
            '\u{000c}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            control if control <= '\u{001f}' => {
                let code = control as usize;
                output.push_str("\\u00");
                output.push(HEX[code >> 4] as char);
                output.push(HEX[code & 0x0f] as char);
            }
            other => output.push(other),
        }
    }
    output
}

#[cfg(test)]
#[path = "output_test.rs"]
mod output_test;
