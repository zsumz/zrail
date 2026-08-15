//! JSON escaping remains standards-compliant without another CLI dependency.

use crate::app::error::CliError;

use super::{OutputFormat, json_escape, render_error};

#[test]
fn json_strings_preserve_unicode_and_escape_controls() {
    assert_eq!(json_escape("zrail é\n\"\\"), "zrail é\\n\\\"\\\\");
    assert_eq!(json_escape("\u{0001}"), "\\u0001");
}

#[test]
fn json_errors_include_message_and_help() {
    let error = CliError::new("invalid \"contract\"").with_help("run zrail doctor");

    assert_eq!(
        render_error(&error, OutputFormat::Json),
        concat!(
            "{\n",
            "  \"schema\": 1,\n",
            "  \"status\": \"invalid\",\n",
            "  \"error\": \"invalid \\\"contract\\\"\",\n",
            "  \"help\": \"run zrail doctor\"\n",
            "}\n",
        )
    );
}
