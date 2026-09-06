//! Physical input mutations exercise exact raw semantics without executing input programs.

#[path = "images.rs"]
mod images;
#[path = "intervals.rs"]
mod intervals;
#[path = "literals.rs"]
mod literals;

use zrail_core::{RepositoryFilePredicate as Predicate, RepositoryFileRule};

use super::super::metadata::mutations::{Mutation, read_cases};

pub(super) fn cases(rule: &RepositoryFileRule, original: &[u8]) -> Vec<Mutation> {
    if matches!(rule.predicate, Predicate::BytesEqual { .. }) {
        return read_cases();
    }
    let source = std::str::from_utf8(original).expect("frozen UTF-8 source");
    let mut cases = match &rule.predicate {
        Predicate::Literal(policy) => literals::cases(source, policy),
        Predicate::LiteralOrder { before, after } => intervals::order(source, before, after),
        Predicate::LiteralBetween {
            start,
            end,
            contains,
        } => intervals::between(source, start, end, contains),
        Predicate::LineValuesAllowed { prefix, values, .. } => {
            images::cases(source, prefix, values)
        }
        other => panic!("unsupported frozen qualification predicate: {other:?}"),
    };
    cases.push(Mutation {
        name: "invalid-utf8".into(),
        bytes: Some(vec![0xff]),
        directory: false,
        diagnostic: Some("REP-FILE-006"),
    });
    cases
}

fn text(name: &str, source: String, diagnostic: Option<&'static str>) -> Mutation {
    Mutation {
        name: name.into(),
        bytes: Some(source.into_bytes()),
        directory: false,
        diagnostic,
    }
}

fn toggled(text: &str) -> String {
    text.chars()
        .map(|character| {
            if character.is_ascii_lowercase() {
                character.to_ascii_uppercase()
            } else {
                character.to_ascii_lowercase()
            }
        })
        .collect()
}

fn split(text: &str) -> String {
    let (first, rest) = text.split_at(text.chars().next().unwrap().len_utf8());
    format!("{first}\u{200b}{rest}")
}
