//! Lexical macro globs resolve aliases without escaping their enclosing scope.

use syn::visit::Visit as _;

use super::FactVisitor;
use crate::source::imports::ImportMap;

#[test]
fn local_glob_candidates_resolve_outer_aliases_only_inside_their_scope() {
    let file = syn::parse_file(
        "use dependency as alias;
         fn scoped() { use alias::*; reviewed!(); }
         fn outside() { reviewed!(); }",
    )
    .expect("parse scoped glob fixture");
    let imports = ImportMap::from_file(&file);
    let mut visitor = FactVisitor::new(&imports);

    visitor.visit_file(&file);

    assert_eq!(
        visitor
            .macro_expansions
            .iter()
            .filter(|expansion| expansion
                .candidates
                .iter()
                .any(|candidate| candidate.observation.name == "dependency::reviewed"))
            .count(),
        1
    );
}

#[test]
fn absolute_macro_paths_bypass_lexical_aliases_and_globs() {
    let file = syn::parse_file(
        "use replacement as dependency;
         fn run() { use local::*; ::dependency::reviewed!(); }",
    )
    .expect("parse absolute macro fixture");
    let imports = ImportMap::from_file(&file);
    let mut visitor = FactVisitor::new(&imports);

    visitor.visit_file(&file);

    let [expansion] = visitor.macro_expansions.as_slice() else {
        panic!("expected one macro invocation");
    };
    assert!(expansion.absolute_path);
    assert_eq!(expansion.candidates.len(), 1);
    assert_eq!(
        expansion.candidates[0].observation.name,
        "dependency::reviewed"
    );
}
