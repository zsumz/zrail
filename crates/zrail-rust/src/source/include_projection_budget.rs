//! One transactional budget bounds include projection across the repository.

use super::{RustFileFacts, parse};

pub(super) const MAX_TOTAL_PROJECTION_WORK: usize = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ProjectionLimit {
    Work,
    Facts,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ProjectionLimits {
    pub(super) work: usize,
    pub(super) total_facts: usize,
}

impl Default for ProjectionLimits {
    fn default() -> Self {
        Self {
            work: MAX_TOTAL_PROJECTION_WORK,
            total_facts: parse::MAX_TOTAL_SOURCE_FACTS,
        }
    }
}

pub(super) struct ProjectionBudget {
    remaining_work: usize,
    remaining_facts: usize,
}

impl ProjectionBudget {
    pub(super) fn for_files(
        files: &[RustFileFacts],
        limits: ProjectionLimits,
    ) -> Result<Self, ProjectionLimit> {
        let physical_facts = files
            .iter()
            .try_fold(0_usize, |total, file| {
                total.checked_add(parse::fact_count(file))
            })
            .ok_or(ProjectionLimit::Facts)?;
        let remaining_facts = limits
            .total_facts
            .checked_sub(physical_facts)
            .ok_or(ProjectionLimit::Facts)?;
        Ok(Self {
            remaining_work: limits.work,
            remaining_facts,
        })
    }

    pub(super) fn consume_work(&mut self) -> Result<(), ProjectionLimit> {
        self.remaining_work = self
            .remaining_work
            .checked_sub(1)
            .ok_or(ProjectionLimit::Work)?;
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
        *remaining_file_facts = remaining_file_facts
            .checked_sub(1)
            .ok_or(ProjectionLimit::Facts)?;
        Ok(())
    }
}
