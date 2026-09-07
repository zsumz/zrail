//! Byte-exact frozen Rafter line matcher, executed only by trusted qualification tests.

pub(super) fn accepts(source: &str, token: &str) -> bool {
    !source
        .lines()
        .any(|line| matches_forbidden_source(line, token))
}

fn matches_forbidden_source(line: &str, token: &str) -> bool {
    let trimmed = line.trim_start();
    starts_with_token_boundary(trimmed, "use ", token)
        || starts_with_token_boundary(trimmed, "pub use ", token)
        || starts_with_token_boundary(trimmed, "extern crate ", token)
        || line.contains(&format!("{token}::"))
}

fn starts_with_token_boundary(line: &str, prefix: &str, token: &str) -> bool {
    let Some(rest) = line.strip_prefix(prefix) else {
        return false;
    };
    let Some(after_token) = rest.strip_prefix(token) else {
        return false;
    };
    after_token
        .chars()
        .next()
        .is_none_or(|ch| matches!(ch, ':' | ';' | ',' | '{' | ' ' | '\t'))
}
