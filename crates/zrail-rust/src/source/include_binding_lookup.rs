//! Binding lookup respects Rust block scopes, module floors, and visibility.

use std::collections::BTreeSet;

use super::{
    BindingKind, IncludeContext, SourceEntry, SyntaxGuard,
    include_binding_resolution::MAX_BINDING_CANDIDATES,
    include_bindings::{BindingSite, IncludeBindings},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::{EffectiveModule, ResolutionUsage, ResolveRequest},
};

impl IncludeBindings {
    pub(super) fn alias_sites(
        &self,
        request: &ResolveRequest<'_>,
        name: &str,
        suffix: &str,
        context: &SyntaxGuard,
        module: &EffectiveModule,
        budget: &mut ProjectionBudget,
    ) -> Result<Vec<BindingSite>, ProjectionLimit> {
        let instance = request.instance;
        let scope = request.scope;
        let mode = &request.mode;
        let Some(source) = self.instances.get(instance) else {
            return Ok(Vec::new());
        };
        let floor = if mode.exact_scope() {
            scope.len()
        } else {
            self.lexical_floor(&source.file, scope, budget)?
        };
        let mut selected = Vec::new();
        let mut depth = None;
        for binding in self
            .files
            .get(&source.file)
            .and_then(|bindings| bindings.named.get(name))
            .into_iter()
            .flatten()
        {
            budget.consume_work()?;
            if mode.extern_root()
                && !matches!(
                    binding.anchor,
                    super::BindingAnchor::ExternRoot | super::BindingAnchor::CrateRoot
                )
            {
                continue;
            }
            let visible = if mode.exact_scope() {
                scope == binding.lexical_scope
            } else {
                binding.lexical_scope.len() >= floor && scope.starts_with(&binding.lexical_scope)
            };
            let availability = binding
                .guard
                .availability_for_domain(context, &source.domain);
            if availability.is_available() && visible {
                let mut binding = binding.clone();
                if availability == super::GuardAvailability::Possible {
                    binding.quality = zrail_core::AnalysisQuality::Unresolved;
                }
                binding.quality = binding.quality.max(self.visibility_quality(
                    &binding.visibility,
                    module,
                    &mode.consumer,
                ));
                select(
                    &mut selected,
                    &mut depth,
                    binding.lexical_scope.len(),
                    BindingSite {
                        binding,
                        instance,
                        module: module.clone(),
                        crossed_include: false,
                    },
                );
            }
        }
        for (edge, child) in self.instances.includes_from(instance) {
            budget.consume_work()?;
            let visible = if mode.exact_scope() {
                scope == edge.parent_scope
            } else {
                edge.parent_scope.len() >= floor && scope.starts_with(&edge.parent_scope)
            };
            if edge.context != IncludeContext::Items || !visible {
                continue;
            }
            for mut site in
                self.exported_alias_sites(*child, name, context, &mut BTreeSet::default(), budget)?
            {
                site.binding.quality = site.binding.quality.max(self.visibility_quality(
                    &site.binding.visibility,
                    &site.module,
                    &mode.consumer,
                ));
                site.crossed_include = true;
                select(&mut selected, &mut depth, edge.parent_scope.len(), site);
            }
        }
        if !selected.is_empty() {
            filter_namespace(&mut selected, suffix, request.usage);
            return Ok(selected);
        }
        if floor == 0
            && let (Some(parent), SourceEntry::Include(edge)) =
                (source.parent, &source.entered_from)
        {
            let Some(parent_module) = self.effective_module(parent, &edge.parent_scope, budget)?
            else {
                return Ok(Vec::new());
            };
            let parent_request = ResolveRequest {
                instance: parent,
                written: request.written,
                scope: &edge.parent_scope,
                depth: request.depth,
                mode: request.mode.clone(),
                usage: request.usage,
                guard: request.guard.clone(),
            };
            let mut inherited = self.alias_sites(
                &parent_request,
                name,
                suffix,
                context,
                &parent_module,
                budget,
            )?;
            for site in &mut inherited {
                site.crossed_include = true;
            }
            return Ok(inherited);
        }
        Ok(Vec::new())
    }
}

fn select(
    selected: &mut Vec<BindingSite>,
    selected_depth: &mut Option<usize>,
    depth: usize,
    site: BindingSite,
) {
    if selected_depth.is_none_or(|current| depth > current) {
        selected.clear();
        *selected_depth = Some(depth);
    }
    if *selected_depth == Some(depth) && selected.len() <= MAX_BINDING_CANDIDATES {
        selected.push(site);
    }
}

fn filter_namespace(sites: &mut Vec<BindingSite>, suffix: &str, usage: ResolutionUsage) {
    if !suffix.is_empty() {
        if usage == ResolutionUsage::ConstructorValue {
            sites.retain(|site| {
                matches!(
                    site.binding.kind,
                    BindingKind::Import | BindingKind::TypeAlias | BindingKind::Module(_)
                )
            });
        } else {
            sites.retain(|site| site.binding.kind != BindingKind::LocalValue);
        }
        return;
    }
    if matches!(
        usage,
        ResolutionUsage::Call | ResolutionUsage::ConstructorValue
    ) {
        sites.retain(|site| {
            matches!(
                site.binding.kind,
                BindingKind::Import | BindingKind::LocalConstructor(_) | BindingKind::LocalValue
            )
        });
    } else if matches!(
        usage,
        ResolutionUsage::Type | ResolutionUsage::OperationType
    ) {
        sites.retain(|site| {
            matches!(
                site.binding.kind,
                BindingKind::Import
                    | BindingKind::TypeAlias
                    | BindingKind::OpaqueAlias
                    | BindingKind::Module(_)
                    | BindingKind::LocalType
                    | BindingKind::LocalConstructor(_)
            )
        });
    }
}
