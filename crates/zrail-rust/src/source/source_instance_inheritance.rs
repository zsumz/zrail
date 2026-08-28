//! Expression includes inherit only the caller bindings Rust places in scope.

use super::super::{IncludeContext, SyntaxGuard};
use super::{SourceEntry, SourceInstance};

#[derive(Default)]
pub(super) struct InheritedBindings {
    pub(super) generic_types: Vec<String>,
    pub(super) prelude_value_shadows: Vec<(String, SyntaxGuard)>,
}

pub(super) fn child_context(
    parent: &SourceInstance,
    entry: &SourceEntry,
) -> Option<(SyntaxGuard, InheritedBindings)> {
    let guard = match entry {
        SourceEntry::Module(edge) => edge.guard.clone(),
        SourceEntry::Include(edge) => edge.guard.clone(),
        SourceEntry::CargoRoot => return None,
    };
    let inherited = match entry {
        SourceEntry::Include(edge) if edge.context == IncludeContext::Expression => {
            let mut generic_types = parent.generic_types.clone();
            generic_types.extend(edge.generic_types.iter().cloned());
            generic_types.sort();
            generic_types.dedup();
            let mut shadows = parent.prelude_value_shadows.clone();
            shadows.extend(edge.prelude_value_shadows.iter().cloned());
            shadows.sort();
            shadows.dedup();
            InheritedBindings {
                generic_types,
                prelude_value_shadows: shadows,
            }
        }
        _ => InheritedBindings::default(),
    };
    Some((guard, inherited))
}
