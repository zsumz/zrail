//! Syntax guards retain canonical cfg identity and exact feature-world availability.

use std::borrow::Borrow;

use super::{
    CfgContext, CfgPredicate, CfgTruth, CompilationDomain, GuardAvailability, SyntaxGuard,
};

impl SyntaxGuard {
    pub(crate) fn canonical_name(&self) -> String {
        match self {
            Self::Ordinary => "ordinary".into(),
            Self::TestOnly => "test-only".into(),
            Self::ProductionOnly => "production-only".into(),
            Self::Never => "never".into(),
            Self::Predicate(predicate) => format!("cfg:{}", predicate.canonical()),
        }
    }

    pub(crate) const fn for_test_only(test_only: bool) -> Self {
        if test_only {
            Self::TestOnly
        } else {
            Self::Ordinary
        }
    }

    pub(crate) fn from_predicate(predicate: CfgPredicate) -> Self {
        match predicate {
            CfgPredicate::True => Self::Ordinary,
            CfgPredicate::False => Self::Never,
            CfgPredicate::Test => Self::TestOnly,
            CfgPredicate::Not(value) if *value == CfgPredicate::Test => Self::ProductionOnly,
            predicate => Self::Predicate(predicate),
        }
    }

    pub(crate) fn available_in(&self, context: impl Borrow<Self>) -> bool {
        self.availability_in(context).is_available()
    }

    pub(crate) fn availability_in(&self, context: impl Borrow<Self>) -> GuardAvailability {
        let context = context.borrow();
        if context.predicate().implies(&self.predicate()) {
            return GuardAvailability::Exact;
        }
        let test = context.test_domain();
        let combined = CfgPredicate::all(vec![self.predicate(), context.predicate()]);
        match combined.evaluate(&CfgContext {
            test,
            active_features: None,
        }) {
            CfgTruth::False => GuardAvailability::Absent,
            CfgTruth::True if !combined.has_unknown_atoms() => GuardAvailability::Exact,
            _ if self == context => GuardAvailability::Exact,
            _ => GuardAvailability::Possible,
        }
    }

    pub(crate) fn availability_for_domain(
        &self,
        context: &Self,
        domain: &CompilationDomain,
    ) -> GuardAvailability {
        let active_features = domain
            .has_exact_features()
            .then_some(&domain.active_features);
        let cfg = CfgContext {
            test: domain.mode.enables_cfg_test(),
            active_features,
        };
        let context_predicate = context.predicate();
        let predicate = self.predicate();
        if context_predicate.evaluate(&cfg) == CfgTruth::False
            || predicate.evaluate(&cfg) == CfgTruth::False
        {
            return GuardAvailability::Absent;
        }
        if context_predicate.implies(&predicate) || predicate.evaluate(&cfg) == CfgTruth::True {
            GuardAvailability::Exact
        } else {
            GuardAvailability::Possible
        }
    }

    pub(crate) fn availability_in_domain(&self, domain: &CompilationDomain) -> GuardAvailability {
        let active_features = domain
            .has_exact_features()
            .then_some(&domain.active_features);
        match self.predicate().evaluate(&CfgContext {
            test: domain.mode.enables_cfg_test(),
            active_features,
        }) {
            CfgTruth::False => GuardAvailability::Absent,
            CfgTruth::True => GuardAvailability::Exact,
            CfgTruth::Unknown => GuardAvailability::Possible,
        }
    }

    pub(crate) fn combine(&self, other: impl Borrow<Self>) -> Self {
        Self::from_predicate(CfgPredicate::all(vec![
            self.predicate(),
            other.borrow().predicate(),
        ]))
    }

    pub(crate) fn overlaps(&self, other: impl Borrow<Self>) -> bool {
        let predicate = CfgPredicate::all(vec![self.predicate(), other.borrow().predicate()]);
        predicate.is_satisfiable().unwrap_or(true)
    }

    pub(crate) fn is_conditional(&self) -> bool {
        self.predicate().has_unknown_atoms()
    }

    pub(crate) fn is_exact(&self) -> bool {
        !self.is_conditional()
    }

    pub(crate) fn is_test_only(&self) -> bool {
        self.predicate().evaluate(&CfgContext {
            test: false,
            active_features: None,
        }) == CfgTruth::False
    }

    pub(crate) fn is_production_applicable(&self) -> bool {
        self.predicate().evaluate(&CfgContext {
            test: false,
            active_features: None,
        }) != CfgTruth::False
    }

    pub(crate) fn predicate(&self) -> CfgPredicate {
        match self {
            Self::Ordinary => CfgPredicate::True,
            Self::TestOnly => CfgPredicate::Test,
            Self::ProductionOnly => CfgPredicate::not(CfgPredicate::Test),
            Self::Never => CfgPredicate::False,
            Self::Predicate(predicate) => predicate.clone(),
        }
    }

    fn test_domain(&self) -> bool {
        self.is_test_only()
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
