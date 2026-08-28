//! Provider queries keep graph storage private and domain-specific.

use crate::source::CompilationDomain;

use super::{ProviderEdges, ProviderGraph};
impl ProviderGraph {
    pub(in crate::source) fn providers(
        &self,
        domain: &CompilationDomain,
        trait_path: &str,
        projection: &[String],
    ) -> Option<&ProviderEdges> {
        self.edges
            .get(&(domain.clone(), trait_path.into(), projection.to_vec()))
    }

    pub(in crate::source) fn complete(
        &self,
        domain: &CompilationDomain,
        trait_path: &str,
        projection: &[String],
    ) -> bool {
        self.declarations
            .get(&(domain.clone(), trait_path.into(), projection.to_vec()))
            .is_some_and(|quality| *quality != zrail_core::AnalysisQuality::Unresolved)
    }
}
