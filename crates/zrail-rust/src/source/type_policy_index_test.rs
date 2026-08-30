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

#[test]
fn replacement_occurrences_are_stored_on_each_declaration_without_span_guessing() {
    let syntax = syn::parse_str::<syn::File>(
        r#"
#[outer]
mod nested {
    #[cfg_attr(feature = "optional", inner)]
    struct Token { epoch: u64 }
    struct Sibling { value: u64 }
}
struct Plain { value: u64 }
"#,
    )
    .unwrap();
    let (facts, _) = collect(&syntax);
    assert_eq!(facts.declarations[0].replacement_macros.len(), 2);
    assert_eq!(facts.declarations[1].replacement_macros.len(), 1);
    assert!(facts.declarations[2].replacement_macros.is_empty());
    let occurrences = &facts.declarations[0].replacement_macros;
    assert_eq!(occurrences[0].span.unwrap().line, 2);
    assert_eq!(occurrences[1].span.unwrap().line, 4);
    assert_eq!(
        occurrences[1].guard.canonical_name(),
        "cfg:feature=\"optional\""
    );
}
