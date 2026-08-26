//! Small operation-identity values shared by extraction helpers.

use std::collections::BTreeMap;

use syn::{Expr, Fields, Item, Path, Type};
use zrail_core::{AnalysisQuality, SourceSpan};

use super::{ObservedFact, SyntaxGuard};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SourceOperationKind {
    TypeConstruction,
    MethodCall,
    FieldReceiverCall,
    FieldRead,
    FieldWrite,
    FieldMutableBorrow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SourceOperationFact {
    pub(crate) kind: SourceOperationKind,
    pub(crate) identity: ObservedFact,
    pub(crate) file_local: bool,
    pub(crate) method: Option<String>,
    pub(crate) place: Option<FieldPlaceFact>,
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
    pub(super) span: Option<SourceSpan>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FieldPlaceFact {
    pub(crate) base_name: String,
    pub(crate) base_quality: AnalysisQuality,
    pub(crate) base_file_local: bool,
    pub(crate) base_span: Option<SourceSpan>,
    pub(crate) fields: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ConstructorForm {
    Named,
    Tuple,
    Unit,
}

#[derive(Clone, Debug)]
pub(super) struct LocalType {
    pub(super) identity: String,
    pub(super) form: ConstructorForm,
    pub(super) fields: BTreeMap<String, Type>,
    pub(super) variants: BTreeMap<String, ConstructorForm>,
}

pub(super) type LocalTypes = BTreeMap<String, LocalType>;

pub(super) fn local_type(item: &Item, prefix: &str) -> Option<(String, LocalType)> {
    let (name, form, fields, variants) = match item {
        Item::Struct(item) => (
            item.ident.to_string(),
            fields_form(&item.fields),
            named_fields(&item.fields),
            BTreeMap::new(),
        ),
        Item::Enum(item) => (
            item.ident.to_string(),
            ConstructorForm::Named,
            BTreeMap::new(),
            item.variants
                .iter()
                .map(|variant| (variant.ident.to_string(), fields_form(&variant.fields)))
                .collect(),
        ),
        _ => return None,
    };
    let identity = if prefix.is_empty() {
        name.clone()
    } else {
        format!("{prefix}::{name}")
    };
    Some((
        name,
        LocalType {
            identity,
            form,
            fields,
            variants,
        },
    ))
}

fn named_fields(fields: &Fields) -> BTreeMap<String, Type> {
    match fields {
        Fields::Named(fields) => fields
            .named
            .iter()
            .filter_map(|field| {
                field
                    .ident
                    .as_ref()
                    .map(|name| (name.to_string(), field.ty.clone()))
            })
            .collect(),
        Fields::Unnamed(_) | Fields::Unit => BTreeMap::new(),
    }
}

fn fields_form(fields: &Fields) -> ConstructorForm {
    match fields {
        Fields::Named(_) => ConstructorForm::Named,
        Fields::Unnamed(_) => ConstructorForm::Tuple,
        Fields::Unit => ConstructorForm::Unit,
    }
}

pub(super) fn append(mut base: TypeIdentity, suffix: impl Iterator<Item = String>) -> TypeIdentity {
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
        span: None,
    }
}

pub(super) fn path_text(path: &Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

pub(super) fn last_segment_looks_constructor(path: &Path) -> bool {
    path.segments.last().is_some_and(|segment| {
        segment.ident == "Self"
            || segment
                .ident
                .to_string()
                .chars()
                .next()
                .is_some_and(char::is_uppercase)
    })
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
