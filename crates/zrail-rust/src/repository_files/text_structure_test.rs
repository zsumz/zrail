//! Escaped observation bytes remain bounded without changing complete violation counts.

use super::{Observation, RepositoryTextNormalization, line_values};

#[test]
fn encoded_line_samples_bound_escaping_and_total_bytes_independently_of_counts() {
    let source = format!(
        "image: {}\n{}",
        "\u{0001}".repeat(4_000),
        format!("image: {}\n", "x".repeat(1_024)).repeat(20)
    );
    let Observation::LineValues {
        selected_count,
        unauthorized_count,
        unauthorized_sample,
        unauthorized_omitted,
    } = line_values(&source, "image: ", &[], RepositoryTextNormalization::None).unwrap()
    else {
        panic!("line values observation")
    };
    assert_eq!((selected_count, unauthorized_count), (21, 21));
    assert_eq!(unauthorized_sample[0].line, 2);
    assert_eq!(unauthorized_sample.len() + unauthorized_omitted, 21);
    assert!(unauthorized_sample.len() < 16);
    assert!(serde_json::to_vec(&unauthorized_sample).unwrap().len() <= 16 * 1024);
}
