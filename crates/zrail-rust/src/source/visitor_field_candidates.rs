//! Field candidates retain exact local declarations and conservative fallback identities.

use syn::{Expr, ExprField, Member, UnOp};
use zrail_core::AnalysisQuality;

use super::super::{
    operation_model::{LocalType, TypeIdentity, unresolved, unwrapped},
    visitor_patterns::PatternInputMode,
    visitor_values::ValueCandidate,
};
use super::{FactVisitor, FieldPlaceFact, SyntaxGuard};

pub(super) struct FieldContext {
    pub(super) identity: TypeIdentity,
    pub(super) place: Option<FieldPlaceFact>,
    pub(super) guard: SyntaxGuard,
}

struct PlaceCandidate {
    base: TypeIdentity,
    fields: Vec<String>,
    guard: SyntaxGuard,
}

pub(super) fn field_contexts(visitor: &FactVisitor<'_>, field: &ExprField) -> Vec<FieldContext> {
    let Member::Named(member) = &field.member else {
        return Vec::new();
    };
    let member = member.to_string();
    let places = place_candidates(visitor, &field.base);
    if !places.is_empty() {
        return places
            .into_iter()
            .map(|candidate| {
                let receiver = project_receiver(visitor, candidate.base.clone(), &candidate.fields);
                let identity = declared_field_identity(visitor, &receiver, &member);
                let mut fields = candidate.fields;
                fields.push(member.clone());
                FieldContext {
                    identity,
                    place: Some(FieldPlaceFact {
                        base_name: candidate.base.name,
                        base_quality: candidate.base.quality,
                        base_file_local: candidate.base.file_local,
                        base_span: candidate.base.span,
                        fields,
                    }),
                    guard: candidate.guard,
                }
            })
            .collect();
    }
    field_receiver_candidates(visitor, &field.base)
        .into_iter()
        .map(|candidate| FieldContext {
            identity: declared_field_identity(visitor, &candidate.identity, &member),
            place: None,
            guard: candidate.guard,
        })
        .collect()
}

pub(super) fn declared_field_identity(
    visitor: &FactVisitor<'_>,
    receiver: &TypeIdentity,
    member: &str,
) -> TypeIdentity {
    let declared =
        local_type(visitor, &receiver.name).is_some_and(|ty| ty.fields.contains_key(member));
    TypeIdentity {
        name: format!("{}::{member}", receiver.name),
        quality: if declared {
            receiver.quality
        } else {
            AnalysisQuality::Unresolved
        },
        file_local: receiver.file_local,
        span: receiver.span,
    }
}

fn place_candidates(visitor: &FactVisitor<'_>, expression: &Expr) -> Vec<PlaceCandidate> {
    match unwrapped(expression) {
        Expr::Path(path) if path.path.is_ident("self") => visitor
            .self_types
            .last()
            .cloned()
            .map(|base| vec![place(base, SyntaxGuard::Ordinary)])
            .unwrap_or_default(),
        Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => visitor
            .local_value_candidates(&path.path.segments[0].ident.to_string())
            .into_iter()
            .map(|candidate| place(candidate.identity, candidate.guard))
            .collect(),
        Expr::Field(field) => {
            let Member::Named(member) = &field.member else {
                return Vec::new();
            };
            place_candidates(visitor, &field.base)
                .into_iter()
                .map(|mut candidate| {
                    candidate.fields.push(member.to_string());
                    candidate
                })
                .collect()
        }
        Expr::Unary(unary) if matches!(unary.op, UnOp::Deref(_)) => {
            place_candidates(visitor, &unary.expr)
        }
        Expr::Cast(cast) => vec![place(visitor.resolve_type(&cast.ty), SyntaxGuard::Ordinary)],
        _ => Vec::new(),
    }
}

fn field_receiver_candidates(visitor: &FactVisitor<'_>, expression: &Expr) -> Vec<ValueCandidate> {
    match unwrapped(expression) {
        Expr::Path(path) if path.path.is_ident("self") => {
            visitor.self_types.last().cloned().map_or_else(
                || vec![unknown_candidate()],
                |identity| {
                    vec![ValueCandidate {
                        identity,
                        guard: SyntaxGuard::Ordinary,
                        input: PatternInputMode::Unresolved,
                    }]
                },
            )
        }
        Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => {
            visitor.local_value_candidates(&path.path.segments[0].ident.to_string())
        }
        Expr::Field(field) => field_value_candidates(visitor, field),
        Expr::Unary(unary) if matches!(unary.op, UnOp::Deref(_)) => {
            field_receiver_candidates(visitor, &unary.expr)
                .into_iter()
                .map(|mut candidate| {
                    candidate.identity.quality = AnalysisQuality::Unresolved;
                    candidate
                })
                .collect()
        }
        Expr::Cast(cast) => vec![ValueCandidate {
            identity: visitor.resolve_type(&cast.ty),
            guard: SyntaxGuard::Ordinary,
            input: visitor.pattern_input_from_type(&cast.ty),
        }],
        _ => vec![unknown_candidate()],
    }
}

fn field_value_candidates(visitor: &FactVisitor<'_>, field: &ExprField) -> Vec<ValueCandidate> {
    let Member::Named(member) = &field.member else {
        return vec![unknown_candidate()];
    };
    field_receiver_candidates(visitor, &field.base)
        .into_iter()
        .map(|candidate| ValueCandidate {
            identity: field_value_type(visitor, &candidate.identity, &member.to_string()),
            guard: candidate.guard,
            input: PatternInputMode::Unresolved,
        })
        .collect()
}

fn project_receiver(
    visitor: &FactVisitor<'_>,
    mut receiver: TypeIdentity,
    fields: &[String],
) -> TypeIdentity {
    for field in fields {
        receiver = field_value_type(visitor, &receiver, field);
    }
    receiver
}

fn field_value_type(
    visitor: &FactVisitor<'_>,
    receiver: &TypeIdentity,
    member: &str,
) -> TypeIdentity {
    let Some(field_type) = local_type(visitor, &receiver.name).and_then(|ty| ty.fields.get(member))
    else {
        return unresolved("<unresolved>");
    };
    let mut identity = visitor.resolve_type(field_type);
    identity.quality = identity.quality.max(receiver.quality);
    identity
}

fn local_type<'a>(visitor: &'a FactVisitor<'_>, identity: &str) -> Option<&'a LocalType> {
    visitor
        .local_types
        .iter()
        .rev()
        .flat_map(|scope| scope.values())
        .find(|local| local.identity == identity)
}

fn place(base: TypeIdentity, guard: SyntaxGuard) -> PlaceCandidate {
    PlaceCandidate {
        base,
        fields: Vec::new(),
        guard,
    }
}

fn unknown_candidate() -> ValueCandidate {
    ValueCandidate {
        identity: unresolved("<unresolved>"),
        guard: SyntaxGuard::Ordinary,
        input: PatternInputMode::Unresolved,
    }
}
