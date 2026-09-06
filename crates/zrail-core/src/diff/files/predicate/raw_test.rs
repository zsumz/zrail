//! Order, interval, and selected-line permissions remain protected under every weakening.

use crate::ChangeKind;
use crate::diff::files::files_test::{configured, kinds, protected};

#[test]
fn removing_order_or_interval_guards_is_a_grant() {
    for predicate in [
        "kind = 'literal-order', before = 'fetch', after = 'gate'",
        "kind = 'literal-between', start = 'gate', end = 'package', contains = 'offline'",
        "kind = 'line-values-allowed', prefix = 'image: ', values = ['pin']",
    ] {
        let before = configured(predicate);
        let mut after = before.clone();
        after.repository.files.clear();
        assert_eq!(kinds(&before, &after), [ChangeKind::Grant]);
        protected(&before, &after);
    }
}

#[test]
fn dropping_raw_boundaries_and_shortening_required_text_are_grants() {
    for (strong, weak) in [
        (
            "kind = 'literal-order', before = 'fetch', after = 'gate'",
            "kind = 'literal', mode = 'contains', text = 'gate'",
        ),
        (
            "kind = 'literal-between', start = 'gate', end = 'package', contains = 'offline'",
            "kind = 'literal', mode = 'contains', text = 'offline'",
        ),
        (
            "kind = 'literal-between', start = 'gate', end = 'package', contains = 'offline=true'",
            "kind = 'literal-between', start = 'gate', end = 'package', contains = 'offline'",
        ),
    ] {
        let strong = configured(strong);
        let weak = configured(weak);
        assert_eq!(kinds(&strong, &weak), [ChangeKind::Grant]);
        assert_eq!(kinds(&weak, &strong), [ChangeKind::Revoke]);
        protected(&strong, &weak);
    }
}

#[test]
fn changed_first_marker_order_and_boundaries_are_never_neutral() {
    let before = configured("kind = 'literal-order', before = 'fetch', after = 'gate'");
    let after = configured("kind = 'literal-order', before = 'gate', after = 'fetch'");
    assert_eq!(kinds(&before, &after), [ChangeKind::Unknown]);
    protected(&before, &after);
    let before = configured(
        "kind = 'literal-between', start = 'gate', end = 'package', contains = 'offline'",
    );
    let after = configured(
        "kind = 'literal-between', start = 'fetch', end = 'package', contains = 'offline'",
    );
    assert_eq!(kinds(&before, &after), [ChangeKind::Unknown]);
    protected(&before, &after);
}

#[test]
fn allowed_value_sets_and_selected_line_scopes_preserve_permission_direction() {
    let before = configured(
        "kind = 'line-values-allowed', prefix = 'image: ', values = ['a', 'b'], normalization = 'trim'",
    );
    let same = configured(
        "kind = 'line-values-allowed', prefix = 'image: ', values = ['b', 'a'], normalization = 'trim'",
    );
    assert!(kinds(&before, &same).is_empty());
    let after = configured(
        "kind = 'line-values-allowed', prefix = 'image: ', values = ['a', 'b', 'new'], normalization = 'trim'",
    );
    assert_eq!(kinds(&before, &after), [ChangeKind::Grant]);
    assert_eq!(kinds(&after, &before), [ChangeKind::Revoke]);
    protected(&before, &after);
    let mut larger = before.clone();
    larger.repository.files[0].include.push("more/**".into());
    assert_eq!(kinds(&larger, &before), [ChangeKind::Grant]);
    protected(&larger, &before);
    let different = configured(
        "kind = 'line-values-allowed', prefix = 'other: ', values = ['a', 'b'], normalization = 'trim'",
    );
    assert_eq!(kinds(&before, &different), [ChangeKind::Unknown]);
    protected(&before, &different);
}
