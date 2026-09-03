//! Allowance names index one or more independently proven provenance entries.

use std::collections::BTreeMap;

use zrail_core::MacroExpansionAllow;

pub(super) struct AllowanceIndex<'a> {
    by_name: BTreeMap<&'a str, Vec<&'a MacroExpansionAllow>>,
}

impl<'a> AllowanceIndex<'a> {
    pub(super) fn new(values: impl IntoIterator<Item = &'a MacroExpansionAllow>) -> Self {
        let mut by_name = BTreeMap::<_, Vec<_>>::new();
        for allowance in values {
            by_name
                .entry(allowance.name.as_str())
                .or_default()
                .push(allowance);
        }
        Self { by_name }
    }

    pub(super) fn contains(&self, name: &str) -> bool {
        self.by_name.contains_key(name)
    }

    pub(super) fn get(&self, name: &str) -> Option<&[&'a MacroExpansionAllow]> {
        self.by_name.get(name).map(Vec::as_slice)
    }
}

#[cfg(test)]
#[path = "allowances_test.rs"]
mod allowances_test;
