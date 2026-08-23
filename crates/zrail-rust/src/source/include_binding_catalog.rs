//! Ordinary bindings are indexed once so projection work scales with matching names.

use std::collections::BTreeMap;

use super::ImportBindingFact;

#[derive(Default)]
pub(super) struct FileBindings {
    pub(super) named: BTreeMap<String, Vec<ImportBindingFact>>,
    pub(super) globs: Vec<ImportBindingFact>,
}

impl FileBindings {
    pub(super) fn collect(bindings: &[ImportBindingFact]) -> Self {
        let mut collected = Self::default();
        for binding in bindings {
            if let Some(name) = &binding.name {
                collected
                    .named
                    .entry(name.clone())
                    .or_default()
                    .push(binding.clone());
            } else {
                collected.globs.push(binding.clone());
            }
        }
        collected
    }
}
