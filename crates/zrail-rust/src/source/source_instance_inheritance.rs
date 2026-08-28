//! Expression includes inherit only the caller bindings Rust places in scope.

use super::super::{GuardAvailability, IncludeContext, SyntaxGuard};
use super::{SourceEntry, SourceInstance};

#[derive(Default)]
pub(super) struct InheritedBindings {
    pub(super) generic_types: Vec<String>,
    pub(super) generic_values: Vec<String>,
    pub(super) value_shadows: Vec<(String, SyntaxGuard)>,
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
            let mut generic_values = parent.generic_values.clone();
            generic_values.extend(edge.generic_values.iter().cloned());
            generic_values.sort();
            generic_values.dedup();
            let mut shadows = parent.value_shadows.clone();
            shadows.extend(edge.value_shadows.iter().cloned());
            shadows.sort();
            shadows.dedup();
            InheritedBindings {
                generic_types,
                generic_values,
                value_shadows: shadows,
            }
        }
        _ => InheritedBindings::default(),
    };
    Some((guard, inherited))
}

impl SourceInstance {
    pub(crate) fn value_shadow_availability(
        &self,
        written: &str,
        guard: &SyntaxGuard,
    ) -> GuardAvailability {
        if written.starts_with("::") || written.contains("::") {
            return GuardAvailability::Absent;
        }
        let root = written.strip_prefix("r#").unwrap_or(written);
        let context = self.guard.combine(guard);
        self.value_shadows
            .iter()
            .filter(|(name, _)| name == root)
            .fold(GuardAvailability::Absent, |current, (_, shadow)| {
                merge(
                    current,
                    shadow.availability_for_domain(&context, &self.domain),
                )
            })
    }
}

const fn merge(left: GuardAvailability, right: GuardAvailability) -> GuardAvailability {
    match (left, right) {
        (GuardAvailability::Exact, _) | (_, GuardAvailability::Exact) => GuardAvailability::Exact,
        (GuardAvailability::Possible, _) | (_, GuardAvailability::Possible) => {
            GuardAvailability::Possible
        }
        _ => GuardAvailability::Absent,
    }
}
