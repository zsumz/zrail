//! Operation identities resolve self, local types, and imported qualified paths.

use syn::{Path, Type, spanned::Spanned};
use zrail_core::AnalysisQuality;

use super::super::fact::source_span;
use super::{
    ConstructorForm, FactVisitor, OperationSubjectOrigin, TypeIdentity, append, path_text,
    unresolved,
};

impl FactVisitor<'_> {
    pub(in crate::source) fn resolve_type(&self, ty: &Type) -> TypeIdentity {
        match ty {
            Type::Path(path) if path.qself.is_none() => self.resolve_identity(&path.path),
            Type::Reference(reference) => self.resolve_type(&reference.elem),
            _ => unresolved("Self"),
        }
    }

    pub(in crate::source) fn resolve_identity(&self, path: &Path) -> TypeIdentity {
        let written = path_text(path);
        if path
            .segments
            .first()
            .is_some_and(|segment| segment.ident == "Self")
        {
            return self.self_identity(path);
        }
        if let Some(local) = self.local_identity(path) {
            return local;
        }
        let (resolved, quality, _, _, _) = self.resolve_text_scoped(&written);
        let qualified = path.leading_colon.is_some() || path.segments.len() > 1;
        let resolved_by_import = resolved != written;
        TypeIdentity {
            name: resolved,
            quality: if qualified || resolved_by_import {
                quality
            } else {
                AnalysisQuality::Unresolved
            },
            file_local: false,
            origin: OperationSubjectOrigin::WrittenPath,
            span: Some(source_span(path.span())),
        }
    }

    fn self_identity(&self, path: &Path) -> TypeIdentity {
        let Some(base) = self.self_types.last() else {
            return unresolved(&path_text(path));
        };
        let suffix = path
            .segments
            .iter()
            .skip(1)
            .map(|segment| segment.ident.to_string());
        let mut identity = append(base.clone(), suffix);
        identity.origin = OperationSubjectOrigin::CurrentSelf;
        identity
    }

    fn local_identity(&self, path: &Path) -> Option<TypeIdentity> {
        let mut segments = path.segments.iter();
        let root = segments.next()?.ident.to_string();
        let local = self
            .local_types
            .iter()
            .rev()
            .find_map(|scope| scope.get(&root))?;
        Some(append(
            TypeIdentity {
                name: local.identity.clone(),
                quality: AnalysisQuality::Exact,
                file_local: true,
                origin: OperationSubjectOrigin::LocalDeclaration,
                span: Some(source_span(path.span())),
            },
            segments.map(|segment| segment.ident.to_string()),
        ))
    }

    pub(super) fn constructor_form(&self, path: &Path) -> Option<(ConstructorForm, bool)> {
        let mut segments = path.segments.iter();
        let root = segments.next()?.ident.to_string();
        if root == "Self" {
            let suffix = segments
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>();
            let form = self.self_types.last().and_then(|identity| {
                self.local_types
                    .iter()
                    .flatten()
                    .find(|(_, local)| local.identity == identity.name)
                    .and_then(|(_, local)| match suffix.as_slice() {
                        [] => Some(local.form),
                        [variant] => local.variants.get(variant).copied(),
                        _ => None,
                    })
            });
            return form.map(|form| (form, true));
        }
        let local = self
            .local_types
            .iter()
            .rev()
            .find_map(|scope| scope.get(&root));
        let suffix = segments
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        match (local, suffix.as_slice()) {
            (Some(local), []) => Some((local.form, true)),
            (Some(local), [variant]) => local
                .variants
                .get(variant)
                .copied()
                .map(|form| (form, true)),
            _ => None,
        }
    }
}
