//! Structural patterns retain governed named-field reads.

use syn::PatStruct;

use super::{FactVisitor, attributes::cfg_guard};

impl FactVisitor<'_> {
    pub(in crate::source) fn record_struct_pattern(&mut self, pattern: &PatStruct) {
        for field in &pattern.fields {
            let guard = self.syntax_guard().combine(cfg_guard(&field.attrs));
            self.record_pattern_field(&pattern.path, &field.member, &guard);
        }
    }

    pub(in crate::source) fn with_pattern_type_paths(&mut self, visit: impl FnOnce(&mut Self)) {
        let previous = std::mem::replace(&mut self.next_path_namespace, super::FactNamespace::Type);
        visit(self);
        self.next_path_namespace = previous;
    }
}
