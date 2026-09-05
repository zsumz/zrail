//! Declarative facades admit vocabulary and thin language entrypoints, not behavior.

use crate::inventory::FileClass;

use super::items;

#[test]
fn facade_declarations_are_declarative() {
    let source = r"
        mod worker /* declaration */;
        use worker::Worker;
        extern crate core;

        pub struct Node { worker: Worker }
        pub enum State { Ready, Stopped }
        pub union Bits { integer: u64, float: f64 }
        pub type NodeId = u64;
        pub const ROOT: NodeId = 0;
    ";

    assert!(violations(FileClass::Facade, source).is_empty());
}

#[test]
fn facade_behavior_remains_implementation() {
    let source = r"
        mod inline { pub fn work() {} }
        pub static STATE: u8 = 0;
        pub trait Service { fn call(&self); }
        pub struct Worker;
        impl Worker { pub fn run(&self) {} }
        pub fn helper() {}
    ";

    assert_eq!(
        violations(FileClass::Facade, source),
        ["inline module", "static", "trait", "impl", "function"]
    );
}

#[test]
fn main_may_be_an_empty_or_single_expression_handoff() {
    let source = r"
        mod app /* declaration */;
        const SUCCESS: i32 = 0;
        fn main() {
            std::process::exit(app::run(std::env::args_os()));
        }
    ";

    assert!(violations(FileClass::EntryPoint, source).is_empty());
    assert!(violations(FileClass::EntryPoint, "fn main() {}").is_empty());
}

#[test]
fn main_rejects_local_behavior_and_non_entry_functions() {
    let source = r"
        fn helper() { app::run() }
        fn main() {
            let code = app::run();
            std::process::exit(code);
        }
    ";

    assert_eq!(
        violations(FileClass::EntryPoint, source),
        ["function", "function"]
    );
}

#[test]
fn main_rejects_behavior_hidden_inside_a_call_argument() {
    let source = r"
        fn main() {
            app::run({
                let configured = true;
                configured
            });
        }
    ";

    assert_eq!(violations(FileClass::EntryPoint, source), ["function"]);
}

#[test]
fn main_rejects_multiple_handoffs_wrapped_as_a_value() {
    for source in [
        "fn main() { (app::start(), app::stop()); }",
        "fn main() { app::run((app::start(), app::stop())); }",
        "fn main() { app::start().then(app::stop()); }",
        "fn main() { app::run(worker.start(), worker.stop()); }",
    ] {
        assert_eq!(violations(FileClass::EntryPoint, source), ["function"]);
    }
}

#[test]
fn all_proc_macro_entrypoint_kinds_may_thinly_delegate() {
    let source = r"
        use proc_macro::TokenStream;
        mod expand /* declaration */;

        #[proc_macro]
        pub fn function(input: TokenStream) -> TokenStream {
            expand::function(input).into()
        }

        #[proc_macro_attribute]
        pub fn attribute(args: TokenStream, input: TokenStream) -> TokenStream {
            expand::attribute(args, input).into()
        }

        #[proc_macro_derive(Reviewed, attributes(reviewed))]
        pub fn derive(input: TokenStream) -> TokenStream {
            expand::derive(input).into()
        }
    ";

    assert!(violations(FileClass::Facade, source).is_empty());
}

#[test]
fn proc_macro_entrypoints_reject_embedded_implementation() {
    let source = r"
        use proc_macro::TokenStream;

        #[proc_macro]
        pub fn expanded(input: TokenStream) -> TokenStream {
            let parsed = input.into();
            expand::function(parsed).into()
        }

        pub fn ordinary(input: TokenStream) -> TokenStream {
            expand::function(input.into()).into()
        }
    ";

    assert_eq!(
        violations(FileClass::Facade, source),
        ["function", "function"]
    );
}

fn violations(class: FileClass, source: &str) -> Vec<String> {
    let syntax = syn::parse_file(source).expect("parse facade fixture");
    items(class, zrail_core::FacadeMode::Declarative, &syntax)
        .into_iter()
        .map(|fact| fact.name)
        .collect()
}

#[test]
fn wiring_reexports_admit_only_the_reviewed_visibility_roots() {
    let syntax = syn::parse_file(
        r"
        pub use crate::a;
        pub(crate) use crate::b;
        pub(super) use crate::c;
        pub(in crate::parent) use crate::d;
        pub(in super::parent) use crate::e;
        pub(self) use crate::f;
        pub(in self::parent) use crate::g;
        use crate::h;
    ",
    )
    .expect("parse import forms");
    assert_eq!(
        items(
            FileClass::Facade,
            zrail_core::FacadeMode::WiringReexports,
            &syntax
        )
        .iter()
        .map(|fact| fact.name.as_str())
        .collect::<Vec<_>>(),
        [
            "non-reexport import",
            "non-reexport import",
            "non-reexport import"
        ]
    );
    assert!(
        items(
            FileClass::Facade,
            zrail_core::FacadeMode::WiringOnly,
            &syntax
        )
        .is_empty()
    );
}

#[test]
fn wiring_modes_reject_even_thin_language_entrypoints() {
    for mode in [
        zrail_core::FacadeMode::WiringOnly,
        zrail_core::FacadeMode::WiringReexports,
    ] {
        for (class, source) in [
            (FileClass::EntryPoint, "fn main() { app::run() }"),
            (
                FileClass::Facade,
                "#[proc_macro] pub fn expand(input: TokenStream) -> TokenStream { worker::run(input) }",
            ),
            (FileClass::Test, "#[test] fn scenario() {}"),
        ] {
            let syntax = syn::parse_file(source).expect("parse language entrypoint");
            assert_eq!(items(class, mode, &syntax).len(), 1);
        }
    }
}
