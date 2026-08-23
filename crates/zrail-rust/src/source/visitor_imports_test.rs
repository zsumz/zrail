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
