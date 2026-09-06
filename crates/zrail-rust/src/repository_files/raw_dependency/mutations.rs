//! Literal mutations distinguish whole-text, complete-line, and independent physical-read claims.

use zrail_core::{RepositoryFilePredicate, RepositoryFileRule, RepositoryLiteralMode as Mode};

use super::super::metadata::mutations::Mutation;

pub(super) fn cases(rule: &RepositoryFileRule, original: &[u8]) -> Vec<Mutation> {
    if matches!(rule.predicate, RepositoryFilePredicate::BytesEqual { .. }) {
        return vec![
            mutation("invalid-utf8", vec![0xff], Some("REP-FILE-005")),
            Mutation {
                name: "missing-input".into(),
                bytes: None,
                directory: false,
                diagnostic: Some("REP-FILE-005"),
            },
            Mutation {
                name: "wrong-entry-kind".into(),
                bytes: None,
                directory: true,
                diagnostic: Some("REP-FILE-005"),
            },
            mutation("empty-readable-input", vec![], None),
        ];
    }
    let RepositoryFilePredicate::Literal(policy) = &rule.predicate else {
        panic!("expected a raw dependency predicate");
    };
    let source = std::str::from_utf8(original).expect("frozen UTF-8 input");
    match policy.mode {
        Mode::LineAbsent => lock_lines(source, &policy.text),
        Mode::LinePresent => attributes(source, &policy.text),
        Mode::Contains | Mode::ExactCount => workflow(source, &policy.text, policy.mode),
        other => panic!("unsupported frozen literal mode: {other:?}"),
    }
}

fn lock_lines(source: &str, text: &str) -> Vec<Mutation> {
    let name = text
        .strip_prefix("name = \"")
        .unwrap()
        .strip_suffix('"')
        .unwrap();
    let mut cases = Vec::new();
    for (label, addition, accepted) in [
        ("forbidden-line", text.to_owned(), false),
        ("duplicate-forbidden-line", format!("{text}\n{text}"), false),
        (
            "padded-forbidden-line",
            format!("\u{2003}\t{text}\t\u{2003}"),
            false,
        ),
        ("unrelated-section", format!("[unrelated]\n{text}"), false),
        (
            "multiline-string",
            format!("note = '''\n{text}\n'''"),
            false,
        ),
        ("similar-name", format!("name = \"{name}-extra\""), true),
        ("commented-line", format!("# {text}"), true),
        ("trailing-comment", format!("{text} # a comment"), true),
        ("changed-case", text.to_uppercase(), true),
        ("nonmatching-spacing", format!("name=\"{name}\""), true),
    ] {
        cases.push(text_case(
            label,
            format!("{source}\n{addition}\n"),
            accepted,
        ));
    }
    cases.push(text_case("empty-input", String::new(), true));
    cases.push(text_case(
        "original-detector-fixture",
        "[[package]]\nname = \"tokio-util-extra\"\n[[package]]\nname = \"bytes\"\n".into(),
        true,
    ));
    cases
}

fn attributes(source: &str, text: &str) -> Vec<Mutation> {
    let mut cases = Vec::new();
    for (label, replacement) in [
        ("missing-line", String::new()),
        ("comment-only", format!("# {text}")),
        ("line-suffix", format!("{text} # suffix")),
        ("changed-case", text.to_uppercase()),
        ("internal-whitespace", text.replace(' ', "  ")),
    ] {
        cases.push(text_case(label, source.replace(text, &replacement), false));
    }
    for (label, value) in [
        (
            "padded-line",
            source.replace(text, &format!("\u{2003}\t{text}\t\u{2003}")),
        ),
        ("crlf", source.replace('\n', "\r\n")),
        ("duplicate-line", format!("{source}\n{text}\n")),
        ("no-final-newline", source.trim_end_matches('\n').to_owned()),
    ] {
        cases.push(text_case(label, value, true));
    }
    cases
}

fn workflow(source: &str, text: &str, mode: Mode) -> Vec<Mutation> {
    let mut cases = Vec::new();
    for (label, replacement) in [
        ("missing-marker", String::new()),
        ("changed-case", text.to_uppercase()),
        ("different-spacing", text.replace(' ', "  ")),
    ] {
        cases.push(text_case(label, source.replace(text, &replacement), false));
    }
    let mut comments = String::new();
    for line in source.lines() {
        comments.push_str("# ");
        comments.push_str(line);
        comments.push('\n');
    }
    cases.push(text_case("comments-only-workflow", comments, true));
    cases.push(text_case(
        "duplicate-marker",
        format!("{source}\n{text}\n"),
        mode == Mode::Contains,
    ));
    if mode == Mode::ExactCount {
        cases.push(text_case(
            "adjacent-markers",
            source.replace(text, &format!("{text}{text}")),
            false,
        ));
        cases.push(text_case(
            "longer-command-with-same-marker",
            source.replace(text, &format!("{text}-extra")),
            true,
        ));
    } else {
        cases.push(text_case(
            "split-marker",
            source.replace(text, &text.replacen(' ', "\n", 1)),
            false,
        ));
        cases.push(text_case(
            "marker-in-unrelated-field",
            format!(
                "{}\nunrelated-description: '{text}'\n",
                source.replace(text, "removed")
            ),
            true,
        ));
    }
    cases
}

fn text_case(name: &str, source: String, accepted: bool) -> Mutation {
    mutation(
        name,
        source.into_bytes(),
        if accepted { None } else { Some("REP-FILE-004") },
    )
}

fn mutation(name: &str, bytes: Vec<u8>, diagnostic: Option<&'static str>) -> Mutation {
    Mutation {
        name: name.into(),
        bytes: Some(bytes),
        directory: false,
        diagnostic,
    }
}
