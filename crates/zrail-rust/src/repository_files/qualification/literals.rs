//! Literal mutations distinguish exact quantity, case, and deliberately unrestricted text scope.

use zrail_core::{RepositoryLiteralMode as Mode, RepositoryLiteralPredicate};

use super::{Mutation, split, text, toggled};

pub(super) fn cases(source: &str, policy: &RepositoryLiteralPredicate) -> Vec<Mutation> {
    let literal = &policy.text;
    let failure = Some("REP-FILE-004");
    match policy.mode {
        Mode::Contains => vec![
            text(
                "omitted-marker",
                source.replace(literal, "RC9_REMOVED"),
                failure,
            ),
            text(
                "changed-case",
                source.replace(literal, &toggled(literal)),
                failure,
            ),
            text(
                "split-marker",
                source.replace(literal, &split(literal)),
                failure,
            ),
            text("duplicate-marker", format!("{source}\n{literal}\n"), None),
            text("comment-wrapped-input", format!("/*\n{source}\n*/\n"), None),
        ],
        Mode::Absent => vec![
            text(
                "forbidden-marker",
                format!("{source}\n{literal}\n"),
                failure,
            ),
            text(
                "commented-forbidden-marker",
                format!("{source}\n# {literal}\n"),
                failure,
            ),
            text(
                "quoted-forbidden-marker",
                format!("{source}\n\"{literal}\"\n"),
                failure,
            ),
            text(
                "changed-case",
                format!("{source}\n{}\n", toggled(literal)),
                None,
            ),
            text(
                "split-marker",
                format!("{source}\n{}\n", split(literal)),
                None,
            ),
            text("comment-wrapped-input", format!("/*\n{source}\n*/\n"), None),
        ],
        Mode::ExactCount => vec![
            text(
                "omitted-occurrence",
                source.replacen(literal, "RC9_REMOVED", 1),
                failure,
            ),
            text(
                "extra-occurrence",
                format!("{source}\n{literal}\n"),
                failure,
            ),
            text(
                "changed-case",
                source.replacen(literal, &toggled(literal), 1),
                failure,
            ),
            text("duplicated-input", format!("{source}\n{source}"), failure),
            text("comment-wrapped-input", format!("/*\n{source}\n*/\n"), None),
        ],
        other => panic!("unsupported frozen literal mode: {other:?}"),
    }
}
