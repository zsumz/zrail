//! Rust module qualifiers navigate effective modules without consuming include edges.

use zrail_core::SourceSpan;

use super::{
    SourceEntry, SourceInstanceId,
    include_bindings::IncludeBindings,
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
};

pub(super) struct QualifiedLocation {
    pub(super) instance: SourceInstanceId,
    pub(super) scope: Vec<SourceSpan>,
    pub(super) written: String,
    pub(super) crossed_include: bool,
    pub(super) unresolved: bool,
}

impl IncludeBindings {
    pub(super) fn resolve_qualifiers(
        &self,
        instance: SourceInstanceId,
        written: &str,
        scope: &[SourceSpan],
        budget: &mut ProjectionBudget,
    ) -> Result<Option<QualifiedLocation>, ProjectionLimit> {
        let segments = written.split("::").collect::<Vec<_>>();
        let mut consumed = 0;
        let mut location = QualifiedLocation {
            instance,
            scope: scope.to_vec(),
            written: String::new(),
            crossed_include: false,
            unresolved: false,
        };
        while let Some(segment) = segments.get(consumed) {
            budget.consume_work()?;
            match *segment {
                "self" => {
                    if self
                        .move_to_current_module(&mut location, budget)?
                        .is_none()
                    {
                        return Ok(Some(unresolved_location(location, written)));
                    }
                    consumed += 1;
                }
                "crate" => {
                    if self.move_to_crate_root(&mut location, budget)?.is_none() {
                        return Ok(Some(unresolved_location(location, written)));
                    }
                    consumed += 1;
                }
                "super" => {
                    if self.move_to_parent_module(&mut location, budget)?.is_none() {
                        return Ok(Some(unresolved_location(location, written)));
                    }
                    consumed += 1;
                }
                _ => break,
            }
        }
        if consumed == 0 {
            return Ok(None);
        }
        location.crossed_include |= self.has_include_ancestor(instance, budget)?;
        if consumed == segments.len() {
            location.written = written.into();
            return Ok(Some(location));
        }
        location.written = segments[consumed..].join("::");
        Ok(Some(location))
    }

    fn has_include_ancestor(
        &self,
        mut instance: SourceInstanceId,
        budget: &mut ProjectionBudget,
    ) -> Result<bool, ProjectionLimit> {
        loop {
            budget.consume_work()?;
            let Some(source) = self.instances.get(instance) else {
                return Ok(false);
            };
            if matches!(source.entered_from, SourceEntry::Include(_)) {
                return Ok(true);
            }
            let Some(parent) = source.parent else {
                return Ok(false);
            };
            instance = parent;
        }
    }

    fn move_to_crate_root(
        &self,
        location: &mut QualifiedLocation,
        budget: &mut ProjectionBudget,
    ) -> Result<Option<()>, ProjectionLimit> {
        loop {
            budget.consume_work()?;
            let Some(source) = self.instances.get(location.instance) else {
                return Ok(None);
            };
            let Some(parent) = source.parent else {
                location.scope.clear();
                return Ok(matches!(source.entered_from, SourceEntry::CargoRoot).then_some(()));
            };
            location.crossed_include |= matches!(source.entered_from, SourceEntry::Include(_));
            location.instance = parent;
        }
    }

    fn move_to_current_module(
        &self,
        location: &mut QualifiedLocation,
        budget: &mut ProjectionBudget,
    ) -> Result<Option<()>, ProjectionLimit> {
        loop {
            budget.consume_work()?;
            let Some(source) = self.instances.get(location.instance) else {
                return Ok(None);
            };
            if let Some(index) = self.inline_module_index(&source.file, &location.scope, budget)? {
                location.scope.truncate(index + 1);
                return Ok(Some(()));
            }
            match (source.parent, &source.entered_from) {
                (Some(parent), SourceEntry::Include(edge)) => {
                    location.instance = parent;
                    location.scope.clone_from(&edge.parent_scope);
                    location.crossed_include = true;
                }
                (_, SourceEntry::CargoRoot | SourceEntry::Module(_)) => {
                    location.scope.clear();
                    return Ok(Some(()));
                }
                _ => return Ok(None),
            }
        }
    }

    fn move_to_parent_module(
        &self,
        location: &mut QualifiedLocation,
        budget: &mut ProjectionBudget,
    ) -> Result<Option<()>, ProjectionLimit> {
        loop {
            budget.consume_work()?;
            let Some(source) = self.instances.get(location.instance) else {
                return Ok(None);
            };
            if let Some(index) = self.inline_module_index(&source.file, &location.scope, budget)? {
                let parent =
                    self.inline_module_index(&source.file, &location.scope[..index], budget)?;
                if let Some(parent) = parent {
                    location.scope.truncate(parent + 1);
                    return Ok(Some(()));
                }
                location.scope.truncate(index);
                return self.move_to_current_module(location, budget);
            }
            match (source.parent, &source.entered_from) {
                (Some(parent), SourceEntry::Include(edge)) => {
                    location.instance = parent;
                    location.scope.clone_from(&edge.parent_scope);
                    location.crossed_include = true;
                }
                (Some(parent), SourceEntry::Module(edge)) => {
                    location.instance = parent;
                    location.scope.clone_from(&edge.parent_scope);
                    self.normalize_module_scope(location, budget)?;
                    return Ok(Some(()));
                }
                _ => return Ok(None),
            }
        }
    }

    fn inline_module_index(
        &self,
        file: &str,
        scope: &[SourceSpan],
        budget: &mut ProjectionBudget,
    ) -> Result<Option<usize>, ProjectionLimit> {
        let Some(modules) = self.inline_module_names.get(file) else {
            return Ok(None);
        };
        for (index, span) in scope.iter().enumerate().rev() {
            budget.consume_work()?;
            if modules.contains_key(span) {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }

    fn normalize_module_scope(
        &self,
        location: &mut QualifiedLocation,
        budget: &mut ProjectionBudget,
    ) -> Result<(), ProjectionLimit> {
        let Some(source) = self.instances.get(location.instance) else {
            location.scope.clear();
            return Ok(());
        };
        if let Some(index) = self.inline_module_index(&source.file, &location.scope, budget)? {
            location.scope.truncate(index + 1);
        } else {
            location.scope.clear();
        }
        Ok(())
    }
}

fn unresolved_location(mut location: QualifiedLocation, written: &str) -> QualifiedLocation {
    location.written = written.into();
    location.unresolved = true;
    location
}
