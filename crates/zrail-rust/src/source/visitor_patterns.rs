//! Pattern bindings retain Rust's explicit and match-ergonomic reference modes.

use syn::{Expr, Member, Pat, PatStruct, Path, Type, UnOp};

use super::{FactVisitor, attributes::cfg_guard};

#[path = "visitor_pattern_bindings.rs"]
mod bindings;

pub(in crate::source) use bindings::binding_input_modes;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::source) enum PatternInputMode {
    Value,
    SharedReference,
    MutableReference,
    Unresolved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::source) enum PatternFieldAccess {
    Read,
    MutableBorrow,
    PossiblyMutableBorrow,
}

impl FactVisitor<'_> {
    pub(in crate::source) fn record_struct_pattern(&mut self, pattern: &PatStruct) {
        let input = self.current_pattern_input();
        for field in &pattern.fields {
            let guard = self.syntax_guard().combine(cfg_guard(&field.attrs));
            let field_input = self.pattern_field_input(&pattern.path, &field.member, input);
            let access = pattern_field_access(&field.pat, field_input);
            self.record_pattern_field(&pattern.path, &field.member, access, &guard);
        }
    }

    pub(in crate::source) fn with_pattern_type_paths(&mut self, visit: impl FnOnce(&mut Self)) {
        let previous = std::mem::replace(&mut self.next_path_namespace, super::FactNamespace::Type);
        visit(self);
        self.next_path_namespace = previous;
    }

    pub(in crate::source) fn current_pattern_input(&self) -> PatternInputMode {
        self.pattern_inputs
            .last()
            .copied()
            .unwrap_or(PatternInputMode::Unresolved)
    }

    pub(in crate::source) fn with_pattern_input(
        &mut self,
        input: PatternInputMode,
        visit: impl FnOnce(&mut Self),
    ) {
        self.pattern_inputs.push(input);
        visit(self);
        self.pattern_inputs.pop();
    }

    pub(in crate::source) fn pattern_input_from_type(&self, ty: &Type) -> PatternInputMode {
        match ty {
            Type::Reference(reference) if reference.mutability.is_some() => {
                PatternInputMode::MutableReference
            }
            Type::Reference(_) => PatternInputMode::SharedReference,
            Type::Group(group) => self.pattern_input_from_type(&group.elem),
            Type::Paren(paren) => self.pattern_input_from_type(&paren.elem),
            Type::Path(path) if self.is_proven_value_path(path) => PatternInputMode::Value,
            Type::Path(_) | Type::Infer(_) | Type::Macro(_) | Type::Verbatim(_) => {
                PatternInputMode::Unresolved
            }
            _ => PatternInputMode::Value,
        }
    }

    pub(in crate::source) fn pattern_input_from_expr(&self, expression: &Expr) -> PatternInputMode {
        match expression {
            Expr::Reference(reference) if reference.mutability.is_some() => {
                PatternInputMode::MutableReference
            }
            Expr::Reference(_) => PatternInputMode::SharedReference,
            Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => {
                let name = path.path.segments[0].ident.to_string();
                common_input(
                    self.local_value_candidates(&name)
                        .into_iter()
                        .map(|candidate| candidate.input),
                )
            }
            Expr::Cast(cast) => self.pattern_input_from_type(&cast.ty),
            Expr::Group(group) => self.pattern_input_from_expr(&group.expr),
            Expr::Paren(paren) => self.pattern_input_from_expr(&paren.expr),
            Expr::Unary(unary) if matches!(unary.op, UnOp::Deref(_)) => PatternInputMode::Value,
            Expr::Array(_) | Expr::Lit(_) | Expr::Struct(_) | Expr::Tuple(_) => {
                PatternInputMode::Value
            }
            _ => PatternInputMode::Unresolved,
        }
    }

    pub(in crate::source) fn pattern_field_input(
        &self,
        path: &Path,
        member: &Member,
        outer: PatternInputMode,
    ) -> PatternInputMode {
        if outer != PatternInputMode::Value {
            return outer;
        }
        let Member::Named(member) = member else {
            return PatternInputMode::Unresolved;
        };
        let identity = self.resolve_identity(path);
        self.local_types
            .iter()
            .rev()
            .flat_map(|scope| scope.values())
            .find(|local| local.identity == identity.name)
            .and_then(|local| local.fields.get(&member.to_string()))
            .map_or(PatternInputMode::Unresolved, |ty| {
                self.pattern_input_from_type(ty)
            })
    }

    fn is_proven_value_path(&self, path: &syn::TypePath) -> bool {
        if path.qself.is_none()
            && path.path.segments.len() == 1
            && is_primitive(&path.path.segments[0].ident.to_string())
        {
            return true;
        }
        let identity = self.resolve_type(&Type::Path(path.clone()));
        self.local_types
            .iter()
            .rev()
            .flat_map(|scope| scope.values())
            .any(|local| local.identity == identity.name)
    }
}

fn pattern_field_access(pattern: &Pat, input: PatternInputMode) -> PatternFieldAccess {
    let modes = binding_input_modes(pattern, input);
    if modes
        .values()
        .any(|mode| *mode == PatternInputMode::MutableReference)
    {
        PatternFieldAccess::MutableBorrow
    } else if modes
        .values()
        .any(|mode| *mode == PatternInputMode::Unresolved)
    {
        PatternFieldAccess::PossiblyMutableBorrow
    } else {
        PatternFieldAccess::Read
    }
}

fn syntactic_input_from_type(ty: &Type) -> PatternInputMode {
    match ty {
        Type::Reference(reference) if reference.mutability.is_some() => {
            PatternInputMode::MutableReference
        }
        Type::Reference(_) => PatternInputMode::SharedReference,
        Type::Group(group) => syntactic_input_from_type(&group.elem),
        Type::Paren(paren) => syntactic_input_from_type(&paren.elem),
        Type::Path(_) | Type::Infer(_) | Type::Macro(_) | Type::Verbatim(_) => {
            PatternInputMode::Unresolved
        }
        _ => PatternInputMode::Value,
    }
}

fn is_primitive(name: &str) -> bool {
    matches!(
        name,
        "bool"
            | "char"
            | "f32"
            | "f64"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "str"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
    )
}

fn common_input(inputs: impl Iterator<Item = PatternInputMode>) -> PatternInputMode {
    inputs
        .reduce(|left, right| {
            if left == right {
                left
            } else {
                PatternInputMode::Unresolved
            }
        })
        .unwrap_or(PatternInputMode::Unresolved)
}
