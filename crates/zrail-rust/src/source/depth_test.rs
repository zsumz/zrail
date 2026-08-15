//! Syntax-depth preflight ignores literals and rejects recursive parser pressure.

use super::{MAX_SYNTAX_DEPTH, check_syntax_depth};

#[test]
fn delimiters_in_comments_and_literals_do_not_consume_depth() {
    let braces = "{".repeat(MAX_SYNTAX_DEPTH + 1);
    let source =
        format!("// {braces}\nconst TEXT: &str = r###\"{braces}\"###;\nconst CHAR: char = '{{';\n");

    assert_eq!(check_syntax_depth(&source), Ok(()));
}

#[test]
fn excessive_syntax_and_comment_nesting_are_rejected() {
    let syntax = format!(
        "{}{}",
        "(".repeat(MAX_SYNTAX_DEPTH + 1),
        ")".repeat(MAX_SYNTAX_DEPTH + 1)
    );
    let comments = format!(
        "{}{}",
        "/*".repeat(MAX_SYNTAX_DEPTH + 1),
        "*/".repeat(MAX_SYNTAX_DEPTH + 1)
    );
    let generics = format!(
        "type Deep = {}u8{};",
        "Vec<".repeat(MAX_SYNTAX_DEPTH + 1),
        ">".repeat(MAX_SYNTAX_DEPTH + 1)
    );

    assert!(check_syntax_depth(&syntax).is_err());
    assert!(check_syntax_depth(&comments).is_err());
    assert!(check_syntax_depth(&generics).is_err());
}
