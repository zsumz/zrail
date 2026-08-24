//! Syntax guards retain exact and possible production/test-domain availability.

use super::{GuardAvailability, SyntaxGuard};

impl SyntaxGuard {
    pub(crate) const fn canonical_name(self) -> &'static str {
        match self {
            Self::Ordinary => "ordinary",
            Self::TestOnly => "test-only",
            Self::ProductionOnly => "production-only",
            Self::Conditional => "conditional",
            Self::ConditionalTestOnly => "conditional-test-only",
            Self::ConditionalProductionOnly => "conditional-production-only",
            Self::Never => "never",
        }
    }

    pub(crate) const fn for_test_only(test_only: bool) -> Self {
        if test_only {
            Self::TestOnly
        } else {
            Self::Ordinary
        }
    }

    pub(crate) const fn available_in(self, context: Self) -> bool {
        self.availability_in(context).is_available()
    }

    pub(crate) const fn availability_in(self, context: Self) -> GuardAvailability {
        let available = match self.domain() {
            Self::Ordinary => true,
            Self::TestOnly => matches!(context.domain(), Self::TestOnly),
            Self::ProductionOnly => {
                matches!(context.domain(), Self::Ordinary | Self::ProductionOnly)
            }
            Self::Never
            | Self::Conditional
            | Self::ConditionalTestOnly
            | Self::ConditionalProductionOnly => false,
        };
        if !available {
            GuardAvailability::Absent
        } else if self.is_conditional() || context.is_conditional() {
            GuardAvailability::Possible
        } else {
            GuardAvailability::Exact
        }
    }

    pub(crate) const fn combine(self, other: Self) -> Self {
        let conditional = self.is_conditional() || other.is_conditional();
        let domain = match (self.domain(), other.domain()) {
            (Self::Never, _) | (_, Self::Never) => Self::Never,
            (Self::Ordinary, value) | (value, Self::Ordinary) => value,
            (Self::TestOnly, Self::TestOnly) => Self::TestOnly,
            (Self::ProductionOnly, Self::ProductionOnly) => Self::ProductionOnly,
            (Self::TestOnly, Self::ProductionOnly) | (Self::ProductionOnly, Self::TestOnly) => {
                Self::Never
            }
            _ => Self::Never,
        };
        match (domain, conditional) {
            (Self::Ordinary, true) => Self::Conditional,
            (Self::TestOnly, true) => Self::ConditionalTestOnly,
            (Self::ProductionOnly, true) => Self::ConditionalProductionOnly,
            (value, _) => value,
        }
    }

    pub(crate) const fn overlaps(self, other: Self) -> bool {
        !matches!(
            (self.domain(), other.domain()),
            (Self::Never, _)
                | (_, Self::Never)
                | (Self::TestOnly, Self::ProductionOnly)
                | (Self::ProductionOnly, Self::TestOnly)
        )
    }

    pub(crate) const fn is_conditional(self) -> bool {
        matches!(
            self,
            Self::Conditional | Self::ConditionalTestOnly | Self::ConditionalProductionOnly
        )
    }

    pub(crate) const fn is_exact(self) -> bool {
        !self.is_conditional()
    }

    pub(crate) const fn is_test_only(self) -> bool {
        matches!(self.domain(), Self::TestOnly)
    }

    pub(crate) const fn is_production_applicable(self) -> bool {
        matches!(self.domain(), Self::Ordinary | Self::ProductionOnly)
    }

    const fn domain(self) -> Self {
        match self {
            Self::Conditional => Self::Ordinary,
            Self::ConditionalTestOnly => Self::TestOnly,
            Self::ConditionalProductionOnly => Self::ProductionOnly,
            value => value,
        }
    }
}

impl GuardAvailability {
    pub(crate) const fn is_available(self) -> bool {
        !matches!(self, Self::Absent)
    }
}

#[cfg(test)]
#[path = "syntax_guard_test.rs"]
mod syntax_guard_test;
