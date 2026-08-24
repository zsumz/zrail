//! Small operation-identity values shared by extraction helpers.

use std::collections::BTreeMap;

use syn::{Expr, ExprField, Fields, Item, Path};
use zrail_core::AnalysisQuality;

use super::{ObservedFact, SyntaxGuard};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SourceOperationKind {
    TypeConstruction,
    MethodCall,
    FieldRead,
    FieldWrite,
    FieldMutableBorrow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SourceOperationFact {
    pub(crate) kind: SourceOperationKind,
    pub(crate) identity: ObservedFact,
    pub(crate) file_local: bool,
}

impl SourceOperationFact {
    pub(super) fn apply_guard(&mut self, guard: SyntaxGuard) {
        self.identity.apply_guard(guard);
    }
}

#[derive(Clone, Debug)]
pub(super) struct TypeIdentity {
    pub(super) name: String,
    pub(super) quality: AnalysisQuality,
    pub(super) file_local: bool,
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
    pub(super) variants: BTreeMap<String, ConstructorForm>,
}

pub(super) type LocalTypes = BTreeMap<String, LocalType>;

pub(super) fn local_type(item: &Item, prefix: &str) -> Option<(String, LocalType)> {
    let (name, form, variants) = match item {
        Item::Struct(item) => (
            item.ident.to_string(),
            fields_form(&item.fields),
            BTreeMap::new(),
        ),
        Item::Enum(item) => (
            item.ident.to_string(),
            ConstructorForm::Named,
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
            variants,
        },
    ))
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

pub(super) fn field_expression(expression: &Expr) -> Option<&ExprField> {
    match unwrapped(expression) {
        Expr::Field(field) => Some(field),
        _ => None,
    }
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
