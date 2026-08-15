//! Fact visitors begin with resolved import declarations and no cfg context.

use zrail_core::AnalysisQuality;

use super::{imports::ImportMap, model::ObservedFact, visitor::FactVisitor};

impl<'a> FactVisitor<'a> {
    pub(super) fn new(imports: &'a ImportMap) -> Self {
        let mut paths = imports
            .declared_paths()
            .into_iter()
            .map(|(path, quality)| ObservedFact {
                name: path.to_owned(),
                span: None,
                quality,
            })
            .collect::<Vec<_>>();
        paths.extend(imports.globs().iter().map(|path| ObservedFact {
            name: path.clone(),
            span: None,
            quality: AnalysisQuality::Conservative,
        }));
        Self {
            imports,
            test_only_context: false,
            paths,
            calls: Vec::new(),
            methods: Vec::new(),
            macros: Vec::new(),
            lint_suppressions: Vec::new(),
            unsafe_constructs: Vec::new(),
            tests: Vec::new(),
            includes: Vec::new(),
            item_macros: Vec::new(),
        }
    }
}
