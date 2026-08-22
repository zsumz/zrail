//! Existing measurable Rust policy debt becomes exact, tightening ratchets.

use std::path::Path;

use zrail_core::{Budget, Contract, TestMode};

use crate::{
    engine::{CheckError, load_model},
    inventory::FileClass,
    source::{Reachability, RustFileFacts},
};

/// Exact measurable Rust policy debt discovered for CLI adoption.
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

/// Registered measurable Rust debt that baseline adoption may ratchet.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BaselineRule {
    /// Per-file source lines above the configured target.
    FileSize,
    /// Inline unit-test modules in production source.
    InlineTests,
    /// Missing module-level responsibility documentation.
    ModuleDocs,
    /// Unsafe constructs forbidden by strict hygiene policy.
    Unsafe,
    /// Lint-suppression attributes governed by strict hygiene policy.
    LintSuppressions,
}

impl BaselineRule {
    /// Complete deterministic registry of baseline-adoptable debt.
    pub const ALL: [Self; 5] = [
        Self::FileSize,
        Self::InlineTests,
        Self::ModuleDocs,
        Self::Unsafe,
        Self::LintSuppressions,
    ];

    /// Stable contract rule name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::FileSize => "rust.file-size",
            Self::InlineTests => "rust.inline-tests",
            Self::ModuleDocs => "rust.module-docs",
            Self::Unsafe => "rust.hygiene.unsafe",
            Self::LintSuppressions => "rust.hygiene.lint-suppressions",
        }
    }

    /// Resolves one exact registered rule name.
    pub fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|rule| rule.name() == name)
    }

    const fn reason(self) -> &'static str {
        match self {
            Self::FileSize => {
                "Observed by `zrail baseline`; reduce this legacy file below its target."
            }
            Self::InlineTests => "Observed by `zrail baseline`; move tests to a sibling module.",
            Self::ModuleDocs => {
                "Observed by `zrail baseline`; add a concise module responsibility statement."
            }
            Self::Unsafe => "Observed by `zrail baseline`; remove or isolate unsafe constructs.",
            Self::LintSuppressions => {
                "Observed by `zrail baseline`; remove or justify lint suppressions."
            }
        }
    }
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

/// Discovers existing measurable Rust debt for CLI baseline adoption.
///
/// `config` identifies the newly rendered contract beneath `root`. The returned
/// plan records exact tightening ratchets without relaxing class-wide hard
/// ceilings; it does not rewrite the contract or lock.
pub fn discover_baseline(root: &Path, config: &Path) -> Result<BaselinePlan, CheckError> {
    discover_baseline_rules(root, config, &BaselineRule::ALL)
}

/// Discovers only the selected registered Rust debt kinds.
pub fn discover_baseline_rules(
    root: &Path,
    config: &Path,
    rules: &[BaselineRule],
) -> Result<BaselinePlan, CheckError> {
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
        for rule in rules {
            if has_debt(*rule, file, contract) {
                plan.ratchets.push(BaselineRatchet {
                    rule: rule.name(),
                    target: file.relative.clone(),
                    reason: rule.reason(),
                });
            }
        }
    }
    plan.ratchets.sort();
    Ok(plan)
}

fn has_debt(rule: BaselineRule, file: &RustFileFacts, contract: &Contract) -> bool {
    if rule == BaselineRule::FileSize {
        return budget(contract, budget_class(file))
            .is_some_and(|budget| file.lines > budget.target);
    }
    if rule == BaselineRule::InlineTests && contract.source.rust.tests != TestMode::Sibling {
        return false;
    }
    crate::rules::count_ratchet::measurement(rule.name(), file, &contract.source.rust)
        .is_some_and(|value| value > 0)
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
