//! Frozen consumer predicates are executed only by trusted differential tests.
//! Origins and exact extracted-body hashes live in the rc9 testkit manifest.

use crate::inventory::FileClass;
use syn::{Item, Visibility};
use zrail_core::FacadeMode;

fn is_declaration(item: &Item) -> bool {
    matches!(item, Item::Use(_)) || matches!(item, Item::Mod(module) if module.content.is_none())
}

fn declarative_facade_item(item: &Item) -> bool {
    match item {
        Item::Mod(module) => module.content.is_none(),
        Item::Use(import) => match &import.vis {
            Visibility::Public(_) => true,
            Visibility::Restricted(restricted) => restricted
                .path
                .segments
                .first()
                .is_some_and(|segment| segment.ident == "crate" || segment.ident == "super"),
            Visibility::Inherited => false,
        },
        _ => false,
    }
}

#[test]
fn frozen_facade_predicates_agree_with_stock_wiring_modes() {
    let fixtures = [
        "",
        "// misleading fn hidden() {}\nmod child;",
        "mod child; pub use child::State;",
        "use child::State;",
        "use child::{State as Alias, self};",
        "pub use child::*;",
        "pub(crate) use child::State;",
        "pub(super) use child::State;",
        "pub(in crate::parent) use child::State;",
        "pub(self) use child::State;",
        "pub(in self::parent) use child::State;",
        "#[cfg(test)] mod child;",
        "#[cfg(any())] struct Unreachable;",
        "pub struct State;",
        "pub enum State { Ready }",
        "pub union State { value: u8 }",
        "pub const VALUE: u8 = 0;",
        "pub static VALUE: u8 = 0;",
        "pub type State = u8;",
        "pub trait State {}",
        "fn hidden() {}",
        "impl State {}",
        "mod child {}",
        "extern crate core;",
        "macro_rules! hidden { () => {} }",
        "include!(\"child.rs\");",
        "#[test] fn scenario() {}",
        "#[proc_macro] fn expand() {}",
    ];
    for source in fixtures {
        let syntax = syn::parse_file(source).expect("parse differential fixture");
        for (mode, expected) in [
            (
                FacadeMode::WiringOnly,
                syntax.items.iter().all(is_declaration),
            ),
            (
                FacadeMode::WiringReexports,
                syntax.items.iter().all(declarative_facade_item),
            ),
        ] {
            assert_eq!(
                super::items(FileClass::Facade, mode, &syntax).is_empty(),
                expected,
                "unaccounted legacy/stock facade difference: {mode:?}, {source}"
            );
        }
    }
}
