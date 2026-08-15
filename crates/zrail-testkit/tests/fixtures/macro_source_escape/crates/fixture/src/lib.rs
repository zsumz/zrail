//! Macro source escape fixture facade.

macro_rules! hidden_module {
    () => {
        #[path = "../../../../hidden.rs"]
        mod hidden;
    };
}

hidden_module!();
