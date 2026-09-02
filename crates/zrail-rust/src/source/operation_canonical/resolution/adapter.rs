//! One resolver instance owns the shared projection budget and completed-result cache.

use super::super::super::include_projection_budget::{ProjectionBudget, ProjectionLimit};
use super::{Cache, Request, Resolution, resolve_request};

pub(in crate::source) struct Resolver<'a> {
    budget: &'a mut ProjectionBudget,
    cache: Cache,
}

impl<'a> Resolver<'a> {
    pub(in crate::source) fn new(budget: &'a mut ProjectionBudget) -> Self {
        Self {
            budget,
            cache: Cache::default(),
        }
    }

    pub(in crate::source) fn resolve(
        &mut self,
        request: Request<'_>,
    ) -> Result<Resolution, ProjectionLimit> {
        resolve_request(request, self.budget, &mut self.cache)
    }

    pub(in crate::source) fn retain_fact(
        &mut self,
        remaining_file_facts: &mut usize,
    ) -> Result<(), ProjectionLimit> {
        self.budget.retain_fact(remaining_file_facts)
    }
}
