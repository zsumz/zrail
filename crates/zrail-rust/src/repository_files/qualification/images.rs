//! Exact image suffixes preserve raw line selection, including its deliberate zero-match cases.

use super::{Mutation, text, toggled};

pub(super) fn cases(source: &str, prefix: &str, values: &[String]) -> Vec<Mutation> {
    let failure = Some("REP-FILE-008");
    let pin = &values[0];
    vec![
        text(
            "forbidden-value",
            format!("{source}\n{prefix}unreviewed\n"),
            failure,
        ),
        text(
            "indented-forbidden-value",
            format!("{source}\n  {prefix}unreviewed\n"),
            failure,
        ),
        text(
            "raw-block-literal",
            format!("{source}\nnote: |\n  {prefix}unreviewed\n"),
            failure,
        ),
        text(
            "quoted-pin",
            format!("{source}\n{prefix}'{pin}'\n"),
            failure,
        ),
        text(
            "trailing-comment",
            format!("{source}\n{prefix}{pin} # comment\n"),
            failure,
        ),
        text(
            "changed-case",
            format!("{source}\n{prefix}{}\n", toggled(pin)),
            failure,
        ),
        text(
            "empty-unselected-suffix",
            format!("{source}\n{prefix}\n"),
            None,
        ),
        text(
            "nonmatching-spacing",
            format!("{source}\nimage:unreviewed\n"),
            None,
        ),
        text(
            "commented-image",
            format!("{source}\n# {prefix}unreviewed\n"),
            None,
        ),
        text(
            "irrelevant-prefix",
            format!("{source}\nnote: {prefix}unreviewed\n"),
            None,
        ),
        text(
            "unicode-padding",
            format!("{source}\n\u{2003}\t{prefix}{pin}\t\u{2003}\n"),
            None,
        ),
        text(
            "duplicate-reviewed-image",
            format!("{source}\n{prefix}{pin}\n{prefix}{pin}\n"),
            None,
        ),
        text(
            "alternate-reviewed-image",
            format!("{source}\n{prefix}{}\n", values[1]),
            None,
        ),
        text("crlf", source.replace('\n', "\r\n"), None),
        text("zero-selected-lines", String::new(), None),
        text(
            "omitted-unauthorized-samples",
            format!("{source}\n{}", format!("{prefix}unreviewed\n").repeat(21)),
            failure,
        ),
    ]
}
