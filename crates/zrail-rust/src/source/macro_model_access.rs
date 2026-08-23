//! Expansion facts expose their written observation to shared fact consumers.

use super::{MacroExpansionFact, ObservedFact};

impl std::ops::Deref for MacroExpansionFact {
    type Target = ObservedFact;

    fn deref(&self) -> &Self::Target {
        &self.observation
    }
}

impl std::ops::DerefMut for MacroExpansionFact {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.observation
    }
}
