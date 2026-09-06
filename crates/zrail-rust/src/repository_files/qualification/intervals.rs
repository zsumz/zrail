//! First-marker mutations reject later repairs and matching text outside the selected interval.

use super::{Mutation, text};

pub(super) fn order(source: &str, before: &str, after: &str) -> Vec<Mutation> {
    let failure = Some("REP-FILE-008");
    let reversed = source
        .replacen(before, "RC9_FIRST_MARKER", 1)
        .replacen(after, before, 1)
        .replacen("RC9_FIRST_MARKER", after, 1);
    vec![
        text(
            "missing-before",
            source.replace(before, "RC9_REMOVED"),
            failure,
        ),
        text(
            "missing-after",
            source.replace(after, "RC9_REMOVED"),
            failure,
        ),
        text("reversed-first-markers", reversed.clone(), failure),
        text(
            "earlier-comment-after",
            format!("# {after}\n{source}"),
            failure,
        ),
        text(
            "later-ordered-pair-cannot-repair",
            format!("{reversed}\n{before}\n{after}\n"),
            failure,
        ),
        text("comment-wrapped-input", format!("/*\n{source}\n*/\n"), None),
        text("unicode-prefix", format!("\u{1f680}\n{source}"), None),
        text(
            "later-duplicates",
            format!("{source}\n{before}\n{after}\n"),
            None,
        ),
    ]
}

pub(super) fn between(source: &str, start: &str, end: &str, contains: &str) -> Vec<Mutation> {
    let failure = Some("REP-FILE-008");
    let removed = source.replace(contains, "RC9_REMOVED");
    vec![
        text("missing-interval-marker", removed.clone(), failure),
        text(
            "marker-only-before-interval",
            format!("# {contains}\n{removed}"),
            failure,
        ),
        text(
            "marker-only-after-interval",
            format!("{removed}\n{contains}\n"),
            failure,
        ),
        text(
            "later-valid-interval-cannot-repair",
            format!("{removed}\n{start}\n{contains}\n{end}\n"),
            failure,
        ),
        text(
            "missing-start",
            source.replace(start, "RC9_REMOVED"),
            failure,
        ),
        text("missing-end", source.replace(end, "RC9_REMOVED"), failure),
        text("earlier-comment-end", format!("# {end}\n{source}"), failure),
        text("comment-wrapped-input", format!("/*\n{source}\n*/\n"), None),
        text("unicode-prefix", format!("\u{1f680}\n{source}"), None),
        text(
            "repeated-interval-markers",
            source.replacen(
                start,
                &format!("{start}\n{}", format!("{contains}\n").repeat(25)),
                1,
            ),
            None,
        ),
    ]
}
