//! Existing size and inline-test debt becomes exact, tightening ratchets.

use std::path::Path;

use zrail_core::{Budget, Contract, TestMode};

use crate::{
    engine::{CheckError, load_model},
    inventory::FileClass,
    source::{Reachability, RustFileFacts},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaselinePlan {
    pub size: Option<BaselineSize>,
    pub ratchets: Vec<BaselineRatchet>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaselineSize {
    pub facade_hard: usize,
    pub implementation_hard: usize,
    pub test_hard: usize,
    pub auxiliary_hard: usize,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BaselineRatchet {
    pub rule: &'static str,
    pub target: String,
    pub reason: &'static str,
}

impl BaselinePlan {
    pub fn empty() -> Self {
        Self {
            size: None,
            ratchets: Vec::new(),
        }
    }
}

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
            plan.raise_hard(class, file.lines);
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

impl BaselinePlan {
    fn raise_hard(&mut self, class: BudgetClass, value: usize) {
        let Some(size) = &mut self.size else {
            return;
        };
        let hard = match class {
            BudgetClass::Facade => &mut size.facade_hard,
            BudgetClass::Implementation => &mut size.implementation_hard,
            BudgetClass::Test => &mut size.test_hard,
            BudgetClass::Auxiliary => &mut size.auxiliary_hard,
        };
        *hard = (*hard).max(value);
    }
}
