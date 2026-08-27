//! Type-construction syntax records exact forms and conservative candidates.

use syn::{Expr, ExprCall, ExprPath, ExprStruct, spanned::Spanned};
use zrail_core::AnalysisQuality;

use super::{
    ConstructorCandidate, ConstructorForm, FactVisitor, SourceOperationKind, path_text, unresolved,
};
use crate::source::{
    CfgPredicate, FactNamespace, ObservedFact, SyntaxGuard,
    fact::written_fact,
    operation_model::{QualifiedOperationSubject, subject::WrittenOperationSubject, unwrapped},
};

impl FactVisitor<'_> {
    pub(in crate::source) fn record_struct_construction(&mut self, expression: &ExprStruct) {
        let identity = self.resolve_construction_identity(&expression.path);
        self.push_operation(
            SourceOperationKind::TypeConstruction,
            &identity,
            path_text(&expression.path),
            expression
                .path
                .segments
                .last()
                .map(|segment| segment.ident.span()),
            Some(ConstructorForm::Named),
        );
    }

    pub(in crate::source) fn record_call_construction(&mut self, call: &ExprCall) {
        let Expr::Path(callee) = unwrapped(call.func.as_ref()) else {
            return;
        };
        let subject = WrittenOperationSubject::from_expression(callee);
        let written = subject.written();
        let path = subject.construction_path();
        let mut identity = path.as_deref().map_or_else(
            || unresolved(&written),
            |path| self.resolve_construction_identity(path),
        );
        identity.quality = AnalysisQuality::Unresolved;
        let guard = self.constructor_candidate_guard(&callee.path, subject.is_qualified());
        if guard.predicate().is_satisfiable() == Some(false) {
            return;
        }
        self.push_guarded_constructor_candidate(
            &identity,
            written,
            callee
                .path
                .segments
                .last()
                .map(|segment| segment.ident.span()),
            &guard,
            ConstructorCandidate {
                kind: SourceOperationKind::TypeConstruction,
                form: ConstructorForm::Tuple,
                proven: false,
                qualified_subject: self.qualified_subject(subject),
            },
        );
    }

    pub(in crate::source) fn record_path_construction(&mut self, expression: &ExprPath) {
        if self
            .constructor_path_exclusions
            .contains(&crate::source::fact::source_span(expression.path.span()))
        {
            return;
        }
        let subject = WrittenOperationSubject::from_expression(expression);
        let written = subject.written();
        let path = subject.construction_path();
        let mut identity = path.as_deref().map_or_else(
            || unresolved(&written),
            |path| self.resolve_construction_identity(path),
        );
        identity.quality = AnalysisQuality::Unresolved;
        let guard = self.constructor_candidate_guard(&expression.path, subject.is_qualified());
        if guard.predicate().is_satisfiable() == Some(false) {
            return;
        }
        self.push_guarded_constructor_candidate(
            &identity,
            written,
            expression
                .path
                .segments
                .last()
                .map(|segment| segment.ident.span()),
            &guard,
            ConstructorCandidate {
                kind: SourceOperationKind::ConstructorCapability,
                form: ConstructorForm::Unknown,
                proven: false,
                qualified_subject: self.qualified_subject(subject),
            },
        );
    }

    fn constructor_candidate_guard(&self, path: &syn::Path, qualified: bool) -> SyntaxGuard {
        let guard = self.syntax_guard();
        if qualified || path.leading_colon.is_some() || path.segments.len() != 1 {
            return guard;
        }
        let name = path.segments[0].ident.to_string();
        guard.combine(SyntaxGuard::from_predicate(CfgPredicate::not(
            self.local_value_shadow_guard(&name).predicate(),
        )))
    }

    fn qualified_subject(
        &self,
        subject: WrittenOperationSubject<'_>,
    ) -> Option<QualifiedOperationSubject> {
        subject.is_qualified().then(|| QualifiedOperationSubject {
            lookup: subject
                .construction_path()
                .as_deref()
                .map_or_else(|| subject.written(), path_text),
            explicit_trait: subject.is_trait_qualified(),
            direct_trait_item: subject.is_trait_qualified()
                && subject.associated_segments() == Some(1),
            trait_identity: subject
                .explicit_trait_path()
                .map(|path| self.qualified_trait_fact(&path)),
            force_unresolved: subject.force_unresolved(&self.generic_types),
        })
    }

    fn qualified_trait_fact(&self, path: &syn::Path) -> ObservedFact {
        let written = path_text(path);
        let mut fact = written_fact(
            written.clone(),
            written,
            path.span(),
            AnalysisQuality::Exact,
            &self.lexical_scope,
        );
        fact.guard = self.syntax_guard();
        fact.namespace = FactNamespace::Type;
        fact
    }
}
