//! Scoped glob collection is direct, guarded, and independent of nested modules.

use super::collect;
use crate::source::SyntaxGuard;

#[test]
fn direct_grouped_globs_retain_their_guard() {
    let file =
        syn::parse_file("#[cfg(test)] use super::{one::*, two::*};").expect("parse guarded globs");

    let globs = collect(file.items.iter());

    assert_eq!(globs["super::one"], SyntaxGuard::TestOnly);
    assert_eq!(globs["super::two"], SyntaxGuard::TestOnly);
}

#[test]
fn nested_module_globs_do_not_escape_their_scope() {
    let file = syn::parse_file("use dependency::*; mod tests { use super::*; }")
        .expect("parse nested globs");

    let globs = collect(file.items.iter());

    assert_eq!(globs.len(), 1);
    assert_eq!(globs["dependency"], SyntaxGuard::Ordinary);
}
