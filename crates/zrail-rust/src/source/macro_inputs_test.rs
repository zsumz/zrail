//! Macro input analysis remains recursive, conservative, and bounded.

use std::str::FromStr;

use super::{inspect, within_limit};
use crate::source::{imports::ImportMap, visitor::FactVisitor};

#[test]
fn standard_expression_inputs_use_the_normal_source_visitor() {
    let imports = ImportMap::default();
    let mut visitor = FactVisitor::new(&imports);
    let expression = syn::parse_str::<syn::ExprMacro>(
        r#"format!("{}", unsafe { std::process::Command::new("sh") })"#,
    )
    .expect("parse standard macro");

    assert!(!inspect(&mut visitor, &expression.mac, "format"));
    assert!(!visitor.unsafe_constructs.is_empty());
    assert!(
        visitor
            .paths
            .iter()
            .any(|path| path.name.starts_with("std::process"))
    );
}

#[test]
fn compiler_expression_inputs_retain_nested_compile_effects() {
    let imports = ImportMap::default();
    let mut visitor = FactVisitor::new(&imports);
    let expression = syn::parse_str::<syn::ExprMacro>(r#"concat!(env!("HOME"), "/data")"#)
        .expect("parse nested compiler macros");

    assert!(!inspect(&mut visitor, &expression.mac, "concat"));
    assert!(visitor.compile_effects.iter().any(|effect| {
        effect.effect == zrail_core::Effect::CompileEnvironment && effect.invocation.name == "env"
    }));
}

#[test]
fn opaque_scanning_retains_paths_calls_methods_and_nested_macros() {
    let imports = ImportMap::default();
    let mut visitor = FactVisitor::new(&imports);
    let expression = syn::parse_str::<syn::ExprMacro>(
        r#"dsl!(std::process::Command::new("sh"), value.unwrap(), nested!())"#,
    )
    .expect("parse DSL macro");

    assert!(inspect(&mut visitor, &expression.mac, "dsl"));
    assert!(
        visitor
            .paths
            .iter()
            .any(|path| path.name == "std::process::Command::new")
    );
    assert!(
        visitor
            .calls
            .iter()
            .any(|call| call.name == "std::process::Command::new")
    );
    assert!(visitor.methods.iter().any(|method| method.name == "unwrap"));
    assert!(
        visitor
            .macro_expansions
            .iter()
            .any(|macro_| macro_.name == "nested")
    );
}

#[test]
fn oversized_macro_input_never_enters_recursive_parsing() {
    let source = std::iter::repeat_n("value", 8_193)
        .collect::<Vec<_>>()
        .join(",");
    let tokens = proc_macro2::TokenStream::from_str(&source).expect("parse bounded tokens");

    assert!(!within_limit(tokens));
}
