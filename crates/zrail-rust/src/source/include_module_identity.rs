//! Effective modules ignore include edges but retain exact compilation occurrences.

use zrail_core::AnalysisQuality;

use super::{
    BindingVisibility, SourceEntry, SourceInstanceId,
    include_binding_helpers::MAX_RESOLVED_PATH_BYTES,
    include_bindings::IncludeBindings,
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::{EffectiveModule, ModuleBoundary},
};

pub(super) type ModuleIdentityCache = std::cell::RefCell<
    std::collections::BTreeMap<
        (SourceInstanceId, Vec<zrail_core::SourceSpan>),
        Option<EffectiveModule>,
    >,
>;

impl IncludeBindings {
    pub(super) fn effective_module(
        &self,
        instance: SourceInstanceId,
        scope: &[zrail_core::SourceSpan],
        budget: &mut ProjectionBudget,
    ) -> Result<Option<EffectiveModule>, ProjectionLimit> {
        let key = (instance, scope.to_vec());
        if let Some(result) = self.module_cache.borrow().get(&key).cloned() {
            return Ok(result);
        }
        let result = self.uncached_effective_module(instance, scope, budget)?;
        self.module_cache.borrow_mut().insert(key, result.clone());
        Ok(result)
    }

    fn uncached_effective_module(
        &self,
        instance: SourceInstanceId,
        scope: &[zrail_core::SourceSpan],
        budget: &mut ProjectionBudget,
    ) -> Result<Option<EffectiveModule>, ProjectionLimit> {
        budget.consume_work()?;
        let Some(source) = self.instances.get(instance).cloned() else {
            return Ok(None);
        };
        let mut module = match (source.parent, source.entered_from) {
            (None, SourceEntry::CargoRoot) => EffectiveModule {
                root: instance,
                boundaries: Vec::new(),
                names: Vec::new(),
            },
            (Some(parent), SourceEntry::Include(edge)) => {
                let Some(module) = self.effective_module(parent, &edge.parent_scope, budget)?
                else {
                    return Ok(None);
                };
                module
            }
            (Some(parent), SourceEntry::Module(edge)) => {
                let Some(mut module) = self.effective_module(parent, &edge.parent_scope, budget)?
                else {
                    return Ok(None);
                };
                module.boundaries.push(ModuleBoundary::External(instance));
                module.names.push(edge.module_name);
                module
            }
            _ => return Ok(None),
        };
        if let Some(names) = self.inline_module_names.get(&instance) {
            for span in scope {
                budget.consume_work()?;
                if let Some(name) = names.get(span) {
                    module
                        .boundaries
                        .push(ModuleBoundary::Inline(instance, *span));
                    module.names.push(name.clone());
                }
            }
        }
        if prefix_bytes(&module.names) > MAX_RESOLVED_PATH_BYTES {
            return Ok(None);
        }
        Ok(Some(module))
    }

    pub(super) fn lexical_floor(
        &self,
        instance: SourceInstanceId,
        scope: &[zrail_core::SourceSpan],
        budget: &mut ProjectionBudget,
    ) -> Result<usize, ProjectionLimit> {
        let Some(modules) = self.inline_module_names.get(&instance) else {
            return Ok(0);
        };
        for (index, span) in scope.iter().enumerate().rev() {
            budget.consume_work()?;
            if modules.contains_key(span) {
                return Ok(index + 1);
            }
        }
        Ok(0)
    }

    pub(super) fn visibility_quality(
        &self,
        visibility: &BindingVisibility,
        declaration: &EffectiveModule,
        consumer: &EffectiveModule,
    ) -> AnalysisQuality {
        if matches!(visibility, BindingVisibility::Public) {
            return if declaration.root == consumer.root {
                AnalysisQuality::Exact
            } else {
                AnalysisQuality::Unresolved
            };
        }
        let allowed = match visibility {
            BindingVisibility::Public => return AnalysisQuality::Exact,
            BindingVisibility::Private => Some(declaration.clone()),
            BindingVisibility::Restricted(path) => visibility_anchor(
                declaration,
                path,
                self.instances
                    .get(declaration.root)
                    .is_some_and(|source| source.domain.edition == "2015"),
            ),
        };
        if allowed.is_some_and(|allowed| consumer.contains(&allowed)) {
            AnalysisQuality::Exact
        } else {
            AnalysisQuality::Unresolved
        }
    }
}

fn visibility_anchor(
    declaration: &EffectiveModule,
    path: &[String],
    edition_2015: bool,
) -> Option<EffectiveModule> {
    let (first, tail) = path.split_first()?;
    let mut target = match first.as_str() {
        "crate" => Vec::new(),
        "self" => declaration.names.clone(),
        "super" => {
            let mut parent = declaration.names.clone();
            parent.pop()?;
            parent
        }
        name if edition_2015 => vec![name.into()],
        _ => return None,
    };
    for segment in tail {
        match segment.as_str() {
            "self" => {}
            "super" => {
                target.pop()?;
            }
            name => target.push(name.into()),
        }
    }
    if !declaration.names.starts_with(&target) {
        return None;
    }
    let mut allowed = declaration.clone();
    allowed.names.truncate(target.len());
    allowed.boundaries.truncate(target.len());
    Some(allowed)
}

fn prefix_bytes(names: &[String]) -> usize {
    names
        .iter()
        .map(String::len)
        .sum::<usize>()
        .saturating_add(names.len().saturating_sub(1).saturating_mul(2))
}
