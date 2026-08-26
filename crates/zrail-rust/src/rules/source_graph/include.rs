//! Literal and verified `OUT_DIR` includes become exact source-graph edges.

use crate::source::{IncludeBoundary, IncludeContext, SourceSyntax, join_relative, parent};

use super::{TraversalContext, Walker};

impl Walker<'_> {
    pub(super) fn walk_include(
        &mut self,
        source: &str,
        context: &TraversalContext,
        include: &IncludeBoundary,
    ) {
        let Some(context) = context.with_guard(&include.guard) else {
            return;
        };
        if let Some(output) = &include.out_dir {
            self.walk_out_dir(source, &context, include, output);
            return;
        }
        let Some(relative) = &include.path else {
            let kind = if include.generated {
                "generated include"
            } else {
                "dynamic include"
            };
            self.unresolved(
                source,
                include.span,
                format!("{kind} cannot be resolved exactly: {}", include.expression),
            );
            return;
        };
        match join_relative(&parent(source), relative) {
            Ok(path) => self.follow_include(
                source,
                path,
                &format!("literal include {relative:?}"),
                syntax(include.context),
                include,
                &context,
            ),
            Err(error) => self.resolution_error(
                source,
                include.span,
                &error,
                &format!("literal include {relative:?}"),
            ),
        }
    }

    fn walk_out_dir(
        &mut self,
        source: &str,
        context: &TraversalContext,
        include: &IncludeBoundary,
        output: &str,
    ) {
        let binding = self
            .contract
            .source
            .rust
            .out_dir
            .iter()
            .find(|binding| binding.path == source && binding.output == output);
        let Some(target) = binding.map(|binding| binding.source.clone()) else {
            self.unresolved(
                source,
                include.span,
                format!("OUT_DIR include {output:?} has no verified generated-source binding"),
            );
            return;
        };
        self.seen_out_dir
            .insert((source.to_owned(), output.to_owned()));
        self.follow_include(
            source,
            target,
            &format!("OUT_DIR include {output:?}"),
            syntax(include.context),
            include,
            context,
        );
    }

    pub(super) fn reject_stale_out_dir(&mut self) {
        for binding in &self.contract.source.rust.out_dir {
            if self
                .seen_out_dir
                .contains(&(binding.path.clone(), binding.output.clone()))
            {
                continue;
            }
            self.findings.push(
                zrail_core::Finding::error(
                    "RUST-GRAPH-006",
                    "rust.source-graph.out-dir",
                    "source-graph",
                    format!(
                        "OUT_DIR binding {:?} matches no reachable include in {}",
                        binding.output, binding.path
                    ),
                )
                .at(&binding.path, None)
                .because(&binding.reason),
            );
        }
    }
}

const fn syntax(context: IncludeContext) -> SourceSyntax {
    match context {
        IncludeContext::Items => SourceSyntax::Items,
        IncludeContext::Expression => SourceSyntax::Expression,
    }
}
