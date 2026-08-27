//! Shared syntax operations retain only identities the source can prove.

#[path = "visitor_operations/construction.rs"]
mod construction;
#[path = "visitor_operations/identity.rs"]
mod identity;
#[path = "visitor_operations/record.rs"]
mod record;

use std::collections::BTreeMap;

use syn::{Item, Type};
use zrail_core::AnalysisQuality;

use super::{
    ConstructorForm, FactVisitor, SourceOperationKind,
    operation_model::{
        OperationSubjectOrigin, TypeIdentity, append, local_type, path_text, unresolved,
    },
};

impl FactVisitor<'_> {
    pub(in crate::source) fn with_local_type_scope<'a>(
        &mut self,
        items: impl Iterator<Item = &'a Item>,
        visit: impl FnOnce(&mut Self),
    ) {
        let prefix = self.inline_modules.join("::");
        let types = items
            .filter_map(|item| local_type(item, &prefix))
            .collect::<BTreeMap<_, _>>();
        self.local_types.push(types);
        visit(self);
        self.local_types.pop();
    }

    pub(in crate::source) fn with_inline_module(
        &mut self,
        name: String,
        visit: impl FnOnce(&mut Self),
    ) {
        self.inline_modules.push(name);
        visit(self);
        self.inline_modules.pop();
    }

    pub(in crate::source) fn with_self_type(&mut self, ty: &Type, visit: impl FnOnce(&mut Self)) {
        let anonymous_scope = self.lexical_scope.len() > self.inline_modules.len();
        let identity = if anonymous_scope {
            self.resolve_self_type(ty)
        } else {
            self.resolve_type(ty)
        };
        self.self_types.push(identity);
        visit(self);
        self.self_types.pop();
    }

    pub(in crate::source) fn record_method_operation(&mut self, call: &syn::ExprMethodCall) {
        let identity = TypeIdentity {
            name: call.method.to_string(),
            quality: AnalysisQuality::Exact,
            file_local: false,
            origin: OperationSubjectOrigin::WrittenPath,
            span: Some(super::fact::source_span(call.method.span())),
        };
        self.push_operation(
            SourceOperationKind::MethodCall,
            &identity,
            call.method.to_string(),
            Some(call.method.span()),
            None,
        );
    }
}

#[cfg(test)]
#[path = "visitor_operations_test.rs"]
mod visitor_operations_test;
