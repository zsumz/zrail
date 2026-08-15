//! Markdown evidence-anchor matching coverage.

use super::{contains_anchor, slug};

#[test]
fn recognizes_generated_and_explicit_markdown_anchors() {
    assert!(contains_anchor(
        "### 7.15 Invariants and evidence\n",
        "715-invariants-and-evidence"
    ));
    assert!(contains_anchor(
        "<a id=\"stable-contract\"></a>",
        "stable-contract"
    ));
    assert!(contains_anchor("## Any title {#exact}\n", "exact"));
}

#[test]
fn code_fences_and_inline_html_text_cannot_fake_an_anchor() {
    assert!(!contains_anchor(
        "```markdown\n## Hidden invariant\n```\n",
        "hidden-invariant"
    ));
    assert!(!contains_anchor(
        "The example is `<a id=\"fake\"></a>`.\n",
        "fake"
    ));
}

#[test]
fn slugging_is_deterministic_and_drops_punctuation() {
    assert_eq!(
        slug("Release / qualification: gate"),
        "release-qualification-gate"
    );
}
