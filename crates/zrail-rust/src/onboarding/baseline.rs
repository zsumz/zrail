//! Existing size and inline-test debt becomes exact, tightening ratchets.

use std::path::Path;

use zrail_core::{Budget, Contract, TestMode};

use crate::{
    engine::{CheckError, load_model},
    inventory::FileClass,
    source::{Reachability, RustFileFacts},
};

/// Exact size and test-placement debt discovered for CLI initialization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaselinePlan {
    /// Preserved size ceilings, or `None` when the contract has no size policy.
    pub size: Option<BaselineSize>,
    /// Exact debt entries that can only tighten after initialization.
    pub ratchets: Vec<BaselineRatchet>,
}

/// Class-wide hard line ceilings preserved from the strict contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaselineSize {
    /// The hard ceiling for declarative facade files.
    pub facade_hard: usize,
    /// The hard ceiling for production implementation files.
    pub implementation_hard: usize,
    /// The hard ceiling for test-only files.
    pub test_hard: usize,
    /// The hard ceiling for entry points and auxiliary source.
    pub auxiliary_hard: usize,
}

/// One exact, tightening debt entry discovered during baseline adoption.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BaselineRatchet {
    /// The stable zrail rule name that measures the debt.
    pub rule: &'static str,
    /// The repository-relative source path carrying the debt.
    pub target: String,
    /// The generated explanation included in the initial contract.
    pub reason: &'static str,
}

impl BaselinePlan {
    /// Creates initialization support with no observed debt or size overrides.
    pub fn empty() -> Self {
        Self {
            size: None,
            ratchets: Vec::new(),
        }
    }
}

/// Discovers existing size and inline-test debt for CLI baseline initialization.
///
/// `config` identifies the newly rendered contract beneath `root`. The returned
/// plan records exact tightening ratchets without relaxing class-wide hard
/// ceilings; it does not rewrite the contract or lock.
pub fn discover_baseline(root: &Path, config: &Path) -> Result<BaselinePlan, CheckError> {
    let model = load_model(root, config)?;
    let contract = &model.bundle.contract;
    let mut plan = BaselinePlan {
        size: contract.source.rust.size.as_ref().map(|size| BaselineSize {
            facade_hard: size.facade.hard,
            implementation_hard: size.implementation.hard,
            test_hard: size.test.hard,
            auxiliary_hard: size.auxiliary.hard,
        }),
        ratchets: Vec::new(),
    };
    for file in &model.source.files {
        let class = budget_class(file);
        if budget(contract, class).is_some_and(|budget| file.lines > budget.target) {
            plan.ratchets.push(BaselineRatchet {
                rule: "rust.file-size",
                target: file.relative.clone(),
                reason: "Observed by `zrail init --baseline`; split this source file.",
            });
        }
        if contract.source.rust.tests == TestMode::Sibling
            && file.reachability.is_production()
            && !file.tests.is_empty()
        {
            plan.ratchets.push(BaselineRatchet {
                rule: "rust.inline-tests",
                target: file.relative.clone(),
                reason: "Observed by `zrail init --baseline`; move tests to a sibling module.",
            });
        }
    }
    plan.ratchets.sort();
    Ok(plan)
}

#[derive(Clone, Copy)]
enum BudgetClass {
    Facade,
    Implementation,
    Test,
    Auxiliary,
}

fn budget_class(file: &RustFileFacts) -> BudgetClass {
    if file.class != FileClass::Generated && file.reachability == Reachability::TestOnly {
        return BudgetClass::Test;
    }
    match file.class {
        FileClass::Facade => BudgetClass::Facade,
        FileClass::Implementation | FileClass::Test | FileClass::Generated => {
            BudgetClass::Implementation
        }
        FileClass::Auxiliary | FileClass::EntryPoint => BudgetClass::Auxiliary,
    }
}

fn budget(contract: &Contract, class: BudgetClass) -> Option<Budget> {
    let size = contract.source.rust.size.as_ref()?;
    Some(match class {
        BudgetClass::Facade => size.facade,
        BudgetClass::Implementation => size.implementation,
        BudgetClass::Test => size.test,
        BudgetClass::Auxiliary => size.auxiliary,
    })
}
