//! Validation for the compound field-mutation ownership boundary.

use crate::{OwnerContract, OwnerKind};

use super::super::super::validate_source::valid_rust_path;
use super::{ValidationErrors, validate_exact_operation};

pub(super) fn validate_mutating_method_scope(owner: &OwnerContract, errors: &mut ValidationErrors) {
    if owner.kind != OwnerKind::FieldMutation && !owner.mutating_methods.is_empty() {
        errors.push(format!(
            "{} owner {:?} may not declare mutating_methods",
            kind_name(owner.kind),
            owner.name,
        ));
    }
}

pub(super) fn validate_field_mutation(owner: &OwnerContract, errors: &mut ValidationErrors) {
    validate_exact_operation(owner, "field-mutation", errors);
    if owner.mutating_methods.is_empty() {
        errors.push(format!(
            "field-mutation owner {:?} requires mutating_methods",
            owner.name
        ));
    }
    for method in &owner.mutating_methods {
        if !valid_method_identifier(method) {
            errors.push(format!(
                "field-mutation owner mutating_methods must contain simple Rust identifiers: {method:?}"
            ));
        }
    }
    if !owner
        .mutating_methods
        .windows(2)
        .all(|pair| pair[0] < pair[1])
    {
        errors.push(format!(
            "field-mutation owner {:?} mutating_methods must be sorted and unique",
            owner.name
        ));
    }
}

fn valid_method_identifier(value: &str) -> bool {
    let (identifier, raw) = value
        .strip_prefix("r#")
        .map_or((value, false), |identifier| (identifier, true));
    if identifier == "_"
        || identifier.starts_with("r#")
        || !valid_rust_path(identifier)
        || identifier.contains("::")
    {
        return false;
    }
    if raw {
        !matches!(identifier, "crate" | "self" | "Self" | "super")
    } else {
        !rust_keyword(identifier)
    }
}

fn rust_keyword(value: &str) -> bool {
    matches!(
        value,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "gen"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "try"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
    )
}

const fn kind_name(kind: OwnerKind) -> &'static str {
    match kind {
        OwnerKind::Call => "call",
        OwnerKind::Capability => "capability",
        OwnerKind::Directory => "directory",
        OwnerKind::TypeConstruction => "type-construction",
        OwnerKind::MethodName => "method-name",
        OwnerKind::FieldRead => "field-read",
        OwnerKind::FieldWrite => "field-write",
        OwnerKind::FieldMutableBorrow => "field-mutable-borrow",
        OwnerKind::FieldMutation => "field-mutation",
        OwnerKind::FieldAuthority => "field-authority",
    }
}
