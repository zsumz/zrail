//! Written glob import facts retain nested targets, visibility, guards, and scope.

use syn::visit::Visit;

use super::super::{BindingVisibility, SyntaxGuard, imports::ImportMap, visitor::FactVisitor};

#[test]
fn nested_public_and_block_local_globs_are_distinct() {
    let syntax = syn::parse_file(
        r"
        #[cfg(test)]
        pub(crate) use crate::api::{types::*, values::*};
        fn work() { use crate::local::*; }
        ",
    )
    .expect("parse glob imports");
    let imports = ImportMap::from_file(&syntax);
    let mut visitor = FactVisitor::new(&imports);
    visitor.visit_file(&syntax);

    assert_eq!(visitor.glob_imports.len(), 3);
    for target in ["crate::api::types", "crate::api::values"] {
        let fact = visitor
            .glob_imports
            .iter()
            .find(|fact| fact.target == target)
            .expect("public glob");
        assert_eq!(fact.guard, SyntaxGuard::TestOnly);
        assert_eq!(
            fact.visibility,
            BindingVisibility::Restricted(vec!["crate".into()])
        );
        assert!(fact.lexical_scope.is_empty());
    }
    let local = visitor
        .glob_imports
        .iter()
        .find(|fact| fact.target == "crate::local")
        .expect("local glob");
    assert_eq!(local.visibility, BindingVisibility::Private);
    assert_eq!(local.lexical_scope.len(), 1);
    assert!(local.span.line > 0);
}
