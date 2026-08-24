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
fn standard_macro_inputs_retain_generic_qualified_self_uncertainty() {
    let imports = ImportMap::default();
    let mut visitor = FactVisitor::new(&imports);
    visitor.generic_types.push("process".into());
    let expression = syn::parse_str::<syn::ExprMacro>(r#"vec![<process::Output>::new("sh")]"#)
        .expect("parse generic qualified-self macro input");

    assert!(!inspect(&mut visitor, &expression.mac, "vec"));
    assert!(visitor.calls.iter().any(|call| {
        call.name == "process::Output::new"
            && call.quality == zrail_core::AnalysisQuality::Unresolved
    }));
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
fn matches_inputs_visit_patterns_and_guards() {
    let imports = ImportMap::default();
    let mut visitor = FactVisitor::new(&imports);
    let expression = syn::parse_str::<syn::ExprMacro>(
        "matches!(value, Some(hidden!()) | denied::Variant if effectful_guard())",
    )
    .expect("parse matches input");

    assert!(!inspect(&mut visitor, &expression.mac, "matches"));
    assert!(
        visitor
            .macro_expansions
            .iter()
            .any(|expansion| expansion.name == "hidden")
    );
    assert!(
        visitor
            .paths
            .iter()
            .any(|path| path.name == "denied::Variant")
    );
    assert!(
        visitor
            .calls
            .iter()
            .any(|call| call.name == "effectful_guard")
    );
}

#[test]
fn opaque_scanning_retains_nested_compiler_effect_targets() {
    let imports = ImportMap::default();
    let mut visitor = FactVisitor::new(&imports);
    let expression = syn::parse_str::<syn::ExprMacro>(
        r#"dsl!(env!("HOME"), option_env!("USER"), include_str!("data.txt"))"#,
    )
    .expect("parse opaque compiler effects");

    assert!(inspect(&mut visitor, &expression.mac, "dsl"));
    assert_eq!(
        visitor
            .compile_effects
            .iter()
            .filter(|effect| effect.effect == zrail_core::Effect::CompileEnvironment)
            .count(),
        2
    );
    assert!(visitor.compile_effects.iter().any(|effect| {
        effect.effect == zrail_core::Effect::CompileFilesystem
            && effect.target.as_deref() == Some("data.txt")
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
    assert!(visitor.macros.iter().any(|macro_| macro_.name == "nested"));
}

#[test]
fn opaque_scanning_requires_a_delimited_group_after_a_macro_bang() {
    let imports = ImportMap::default();
    let mut visitor = FactVisitor::new(&imports);
    let expression = syn::parse_str::<syn::ExprMacro>(
        "dsl!(canonical != divergent, !enabled, value ! = other, bare!, call!(), array![], block!{}, path::actual!(value))",
    )
    .expect("parse opaque macro syntax");

    assert!(inspect(&mut visitor, &expression.mac, "dsl"));
    let names = visitor
        .macros
        .iter()
        .map(|macro_| macro_.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, ["call", "array", "block", "path::actual"]);
}

#[test]
fn oversized_macro_input_never_enters_recursive_parsing() {
    let source = std::iter::repeat_n("value", 8_193)
        .collect::<Vec<_>>()
        .join(",");
    let tokens = proc_macro2::TokenStream::from_str(&source).expect("parse bounded tokens");

    assert!(!within_limit(tokens));
}
