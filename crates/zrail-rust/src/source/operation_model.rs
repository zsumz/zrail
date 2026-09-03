//! Small operation-identity values shared by extraction helpers.

#[path = "operation_model/subject.rs"]
pub(super) mod subject;
#[path = "operation_model/syntax_text.rs"]
mod syntax_text;

use std::collections::BTreeMap;

use syn::{Expr, Fields, Item, Type};
use zrail_core::{AnalysisQuality, SourceSpan};

use super::{
    CfgPredicate, ConstructorForm, GenericRootShadow, ObservedFact, RootLookupNamespace,
    SyntaxGuard, attributes::cfg_guard, fact::written_path,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SourceOperationKind {
    TypeConstruction,
    ConstructorCapability,
    MethodCall,
    FieldReceiverCall,
    FieldRead,
    FieldWrite,
    FieldProjectionWrite,
    FieldMutableBorrow,
    FieldProjectionMutableBorrow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SourceOperationFact {
    pub(crate) kind: SourceOperationKind,
    pub(crate) identity: ObservedFact,
    pub(crate) root_lookup: Option<super::RootLookupNamespace>,
    pub(crate) generic_shadow: Option<super::GenericRootShadow>,
    pub(crate) file_local: bool,
    pub(crate) subject_origin: OperationSubjectOrigin,
    pub(crate) construction: Option<ConstructorForm>,
    pub(crate) construction_proven: bool,
    pub(crate) method: Option<String>,
    pub(crate) place: Option<FieldPlaceFact>,
    pub(crate) struct_update: Option<StructUpdateFact>,
    pub(crate) qualified_subject: Option<QualifiedOperationSubject>,
}

impl SourceOperationFact {
    pub(super) fn apply_guard(&mut self, guard: &SyntaxGuard) {
        self.identity.apply_guard(guard);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TypeIdentity {
    pub(super) name: String,
    pub(super) quality: AnalysisQuality,
    pub(super) file_local: bool,
    pub(super) origin: OperationSubjectOrigin,
    pub(super) span: Option<SourceSpan>,
    pub(super) inference: Option<AssociatedReturnInference>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AssociatedReturnInference {
    pub(crate) fact: ObservedFact,
    pub(crate) subject_origin: OperationSubjectOrigin,
    pub(crate) root_lookup: RootLookupNamespace,
    pub(crate) generic_shadow: Option<GenericRootShadow>,
    pub(crate) try_depth: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FieldPlaceFact {
    pub(crate) base_name: String,
    pub(crate) base_quality: AnalysisQuality,
    pub(crate) base_file_local: bool,
    pub(crate) base_origin: OperationSubjectOrigin,
    pub(crate) base_span: Option<SourceSpan>,
    pub(crate) base_inference: Option<AssociatedReturnInference>,
    pub(crate) fields: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StructUpdateFact {
    pub(crate) written: String,
    pub(crate) rest_span: SourceSpan,
    pub(crate) explicit_fields: Vec<StructUpdateField>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StructUpdateField {
    pub(crate) name: String,
    pub(crate) guard: SyntaxGuard,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OperationSubjectOrigin {
    WrittenPath,
    CurrentSelf,
    LocalDeclaration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct QualifiedOperationSubject {
    pub(crate) lookup: String,
    pub(crate) explicit_trait: bool,
    pub(crate) direct_trait_item: bool,
    pub(crate) trait_identity: Option<ObservedFact>,
    pub(crate) force_unresolved: bool,
}

#[derive(Clone, Debug)]
pub(super) struct LocalType {
    pub(super) identity: String,
    pub(super) fields: BTreeMap<String, LocalField>,
}

#[derive(Clone, Debug)]
pub(super) struct LocalField {
    pub(super) ty: Type,
    pub(super) guard: SyntaxGuard,
}

pub(super) type LocalTypes = BTreeMap<String, LocalType>;

pub(super) fn local_type(item: &Item, prefix: &str) -> Option<(String, LocalType)> {
    let (name, fields) = match item {
        Item::Struct(item) => (item.ident.to_string(), named_fields(&item.fields)),
        Item::Enum(item) => (item.ident.to_string(), BTreeMap::new()),
        _ => return None,
    };
    let identity = if prefix.is_empty() {
        name.clone()
    } else {
        format!("{prefix}::{name}")
    };
    Some((name, LocalType { identity, fields }))
}

fn named_fields(fields: &Fields) -> BTreeMap<String, LocalField> {
    let Fields::Named(fields) = fields else {
        return BTreeMap::new();
    };
    let mut named: BTreeMap<String, LocalField> = BTreeMap::new();
    for field in &fields.named {
        let Some(name) = &field.ident else {
            continue;
        };
        let guard = cfg_guard(&field.attrs);
        if let Some(existing) = named.get_mut(&name.to_string()) {
            existing.guard = SyntaxGuard::from_predicate(CfgPredicate::any(vec![
                existing.guard.predicate(),
                guard.predicate(),
            ]));
        } else {
            named.insert(
                name.to_string(),
                LocalField {
                    ty: field.ty.clone(),
                    guard,
                },
            );
        }
    }
    named
}

pub(super) fn append(mut base: TypeIdentity, suffix: impl Iterator<Item = String>) -> TypeIdentity {
    base.inference = None;
    for segment in suffix {
        base.name.push_str("::");
        base.name.push_str(&segment);
    }
    base
}

pub(super) fn unresolved(name: &str) -> TypeIdentity {
    TypeIdentity {
        name: name.into(),
        quality: AnalysisQuality::Unresolved,
        file_local: false,
        origin: OperationSubjectOrigin::WrittenPath,
        span: None,
        inference: None,
    }
}

pub(super) fn path_text(path: &syn::Path) -> String {
    written_path(path)
}

pub(super) fn unwrapped(mut expression: &Expr) -> &Expr {
    loop {
        expression = match expression {
            Expr::Group(group) => &group.expr,
            Expr::Paren(paren) => &paren.expr,
            Expr::Reference(reference) => &reference.expr,
            _ => return expression,
        };
    }
}
