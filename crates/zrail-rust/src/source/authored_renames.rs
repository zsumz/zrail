//! Explicit use-tree membership supplements binding facts that intentionally erase rename syntax.
//!
//! Reuse the parsed file and located-fact representation. Names, same-name renames
//! and underscore aliases remain written syntax, independent of binding resolution.

use syn::{spanned::Spanned, visit::Visit};
use zrail_core::AnalysisQuality;

use crate::source::{ObservedFact, fact::written_fact};

pub(super) fn collect(syntax: &syn::File) -> Vec<ObservedFact> {
    let mut collector = Collector(Vec::new());
    collector.visit_file(syntax);
    collector.0
}

struct Collector(Vec<ObservedFact>);

impl<'ast> Visit<'ast> for Collector {
    fn visit_use_rename(&mut self, rename: &'ast syn::UseRename) {
        if self.0.len() > super::MAX_FACTS_PER_FILE {
            return;
        }
        let source = rename.ident.to_string();
        self.0.push(written_fact(
            source.clone(),
            format!("{source} as {}", rename.rename),
            rename.span(),
            AnalysisQuality::Conservative,
            &[],
        ));
    }
}
