//! Written duplication derives remain violations when provenance degrades.

use zrail_core::{AnalysisQuality, DuplicationTrait, SourceSpan};

use super::derive_quality;

#[test]
fn missing_derive_expansion_fails_closed() {
    let quality = derive_quality(
        &[],
        SourceSpan {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 6,
        },
        DuplicationTrait::Clone,
    );

    assert_eq!(quality, AnalysisQuality::Unresolved);
}
