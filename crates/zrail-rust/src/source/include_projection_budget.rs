//! Input-derived transactional budgets bound only include-connected expansion.

const MIN_PROJECTION_WORK: usize = 1_000_000;
const MIN_PROJECTED_FACTS: usize = 100_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ProjectionLimit {
    Work,
    Facts,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ProjectionLimits {
    pub(super) work: usize,
    pub(super) projected_facts: usize,
}

impl ProjectionLimits {
    pub(super) fn for_input(affected_facts: usize, derived_contexts: usize) -> Self {
        let contextual_work = affected_facts
            .saturating_mul(derived_contexts.max(1))
            .saturating_mul(32);
        Self {
            work: contextual_work
                .saturating_add(affected_facts.saturating_mul(64))
                .saturating_add(derived_contexts.saturating_mul(256))
                .max(MIN_PROJECTION_WORK),
            projected_facts: affected_facts.saturating_mul(8).max(MIN_PROJECTED_FACTS),
        }
    }

    pub(super) fn for_contract(
        affected_facts: usize,
        derived_contexts: usize,
        limits: &zrail_core::AnalysisLimits,
    ) -> Self {
        let derived = Self::for_input(affected_facts, derived_contexts);
        Self {
            work: limits.include_projection_work.unwrap_or(derived.work),
            projected_facts: limits.projected_facts.unwrap_or(derived.projected_facts),
        }
    }
}

pub(super) struct ProjectionBudget {
    remaining_work: usize,
    remaining_facts: usize,
    used_work: usize,
    retained_facts: usize,
}

impl ProjectionBudget {
    pub(super) const fn new(limits: ProjectionLimits) -> Self {
        Self {
            remaining_work: limits.work,
            remaining_facts: limits.projected_facts,
            used_work: 0,
            retained_facts: 0,
        }
    }

    pub(super) fn consume_work(&mut self) -> Result<(), ProjectionLimit> {
        self.remaining_work = self
            .remaining_work
            .checked_sub(1)
            .ok_or(ProjectionLimit::Work)?;
        self.used_work = self.used_work.saturating_add(1);
        Ok(())
    }

    pub(super) fn retain_fact(
        &mut self,
        remaining_file_facts: &mut usize,
    ) -> Result<(), ProjectionLimit> {
        self.consume_work()?;
        self.remaining_facts = self
            .remaining_facts
            .checked_sub(1)
            .ok_or(ProjectionLimit::Facts)?;
        self.retained_facts = self.retained_facts.saturating_add(1);
        *remaining_file_facts = remaining_file_facts
            .checked_sub(1)
            .ok_or(ProjectionLimit::Facts)?;
        Ok(())
    }

    pub(super) const fn used_work(&self) -> usize {
        self.used_work
    }

    pub(super) const fn retained_facts(&self) -> usize {
        self.retained_facts
    }
}
