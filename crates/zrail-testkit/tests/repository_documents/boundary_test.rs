//! Malformed or incomplete documents cannot become trusted partial analysis.

use super::document;

fn incomplete(format: &str, bytes: &[u8], expected: &str) {
    let repository = document(format, "[]", "op = 'present'");
    std::fs::write(repository.0.join("metadata"), bytes).expect("document bytes");
    let error = zrail_rust::build_lock(&repository.0, "zrail.toml".as_ref())
        .expect_err("no partial lock")
        .to_string();
    assert!(
        error.contains("REP-FILE-006")
            && error.contains("repository:file:metadata")
            && error.contains(expected),
        "{error}"
    );
    assert!(zrail_rust::governed_surface_report(&repository.0, "zrail.toml".as_ref()).is_err());
}

#[test]
fn malformed_duplicate_and_non_utf8_documents_fail_closed() {
    for (format, source, expected) in [
        ("toml", "x=1\nx=2", "duplicate key"),
        ("toml", "[package]\nversion = ", "invalid TOML"),
        ("json", "{\"key\":true,\"key\":false}", "duplicate JSON"),
        (
            "json",
            "{\"nested\":{\"key\":1,\"k\\u0065y\":2}}",
            "duplicate JSON",
        ),
        ("json", "{\"key\":1} false", "invalid JSON"),
        ("json", "{\"key\":1,}", "invalid JSON"),
    ] {
        incomplete(format, source.as_bytes(), expected);
    }
    for format in ["toml", "json"] {
        incomplete(format, &[0xff], "not UTF-8");
    }
}

#[test]
fn depth_and_node_limits_reject_uninspected_structure_even_outside_the_selected_key() {
    let array = format!("{}0{}", "[".repeat(66), "]".repeat(66));
    incomplete("json", array.as_bytes(), "safety limit");
    incomplete("toml", format!("nested={array}").as_bytes(), "safety limit");
    let array = format!("[{}0]", "0,".repeat(100_000));
    incomplete("json", array.as_bytes(), "safety limit");
    incomplete("toml", format!("nested={array}").as_bytes(), "safety limit");
}
