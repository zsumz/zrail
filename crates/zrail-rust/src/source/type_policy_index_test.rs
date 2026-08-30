//! Trait implementation facts retain positive and negative source polarity.

use super::collect;
use crate::source::TraitImplPolarity;

#[test]
fn trait_impl_polarity_is_preserved_from_nightly_syntax() {
    let syntax = syn::parse_str::<syn::File>(
        r"#![feature(negative_impls)]
struct Token;
impl Clone for Token {
    fn clone(&self) -> Self { Self }
}
impl !Copy for Token {}
",
    )
    .expect("parse positive and negative trait impl syntax");

    let (facts, _) = collect(&syntax);
    let observed = facts
        .trait_impls
        .iter()
        .map(|implementation| (implementation.trait_hint.as_str(), implementation.polarity))
        .collect::<Vec<_>>();

    assert_eq!(
        observed,
        vec![
            ("Clone", TraitImplPolarity::Positive),
            ("Copy", TraitImplPolarity::Negative),
        ]
    );
}
