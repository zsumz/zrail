//! Frozen trait syntax selects path-type suffixes and preserves both written identities.

pub(super) const SUBJECT: &str = "kind='written-trait-impls',names=['RegisteredTransport','SlotTransport','Source'],implementing_types={Source=['DirectRustlsTransport']}";

pub(super) const CASES: &[(&str, &[(&str, usize)])] = &[
    (
        "impl RegisteredTransport for State {}",
        &[("RegisteredTransport for State", 1)],
    ),
    (
        "impl SlotTransport for Other {}",
        &[("SlotTransport for Other", 1)],
    ),
    (
        "impl Source for DirectRustlsTransport {}",
        &[("Source for DirectRustlsTransport", 1)],
    ),
    ("impl Source for Other {}", &[]),
    ("impl Other for DirectRustlsTransport {}", &[]),
    ("impl DirectRustlsTransport {}", &[]),
    (
        "impl p::RegisteredTransport for q::State {}",
        &[("RegisteredTransport for State", 1)],
    ),
    (
        "impl<T> ::p::Source<T> for q::DirectRustlsTransport<T> {}",
        &[("Source for DirectRustlsTransport", 1)],
    ),
    (
        "impl !Source for DirectRustlsTransport {}",
        &[("Source for DirectRustlsTransport", 1)],
    ),
    (
        "unsafe impl RegisteredTransport for State {}",
        &[("RegisteredTransport for State", 1)],
    ),
    (
        "impl RegisteredTransport for <X as p::Container>::State {}",
        &[("RegisteredTransport for State", 1)],
    ),
    (
        "impl Source for <X as Container>::DirectRustlsTransport {}",
        &[("Source for DirectRustlsTransport", 1)],
    ),
    ("impl RegisteredTransport for &State {}", &[]),
    ("impl RegisteredTransport for (State,) {}", &[]),
    ("impl RegisteredTransport for (State) {}", &[]),
    ("impl RegisteredTransport for _ {}", &[]),
    ("impl RegisteredTransport for *const State {}", &[]),
    ("impl RegisteredTransport for [State] {}", &[]),
    ("impl r#RegisteredTransport for State {}", &[]),
    (
        "impl RegisteredTransport for r#State {}",
        &[("RegisteredTransport for r#State", 1)],
    ),
    ("impl Source for r#DirectRustlsTransport {}", &[]),
    ("impl registeredTransport for State {}", &[]),
    (
        "use p::RegisteredTransport as Rt; impl Rt for State {}",
        &[],
    ),
    (
        "type Alias = DirectRustlsTransport; impl Source for Alias {}",
        &[],
    ),
    (
        "impl RegisteredTransport for Alias {}",
        &[("RegisteredTransport for Alias", 1)],
    ),
    (
        "impl Source for DirectRustlsTransport {} impl Source for DirectRustlsTransport {}",
        &[("Source for DirectRustlsTransport", 2)],
    ),
    (
        "#[cfg(test)] impl Source for DirectRustlsTransport {} #[cfg(not(test))] impl Source for DirectRustlsTransport {} #[cfg(any())] impl Source for DirectRustlsTransport {}",
        &[("Source for DirectRustlsTransport", 3)],
    ),
    (
        "fn f() { impl RegisteredTransport for State {} } mod nested { impl RegisteredTransport for State {} }",
        &[("RegisteredTransport for State", 2)],
    ),
    (
        "#[doc = { impl Source for DirectRustlsTransport {} \"\" }] struct X;",
        &[("Source for DirectRustlsTransport", 1)],
    ),
    (
        "struct X([u8; { impl Source for DirectRustlsTransport {} 0 }]);",
        &[("Source for DirectRustlsTransport", 1)],
    ),
    (
        "impl Other for State { fn f() { impl Source for DirectRustlsTransport {} } }",
        &[("Source for DirectRustlsTransport", 1)],
    ),
    (
        "macro_rules! hidden { () => { impl Source for DirectRustlsTransport {} }; }",
        &[],
    ),
    (
        "fn f() { stringify!(impl Source for DirectRustlsTransport {}); let _s = \"impl Source for DirectRustlsTransport {}\"; /* impl Source for DirectRustlsTransport {} */ }",
        &[],
    ),
];
