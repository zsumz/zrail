//! File and binding scopes wrap Syn's default traversal with resolved context.

use syn::{
    File, Local, PatReference, PatStruct, PatType,
    visit::{self, Visit},
};

use super::super::{
    SyntaxGuard, attributes::cfg_guard, visitor_parts::visitor_patterns::PatternInputMode,
};
use super::FactVisitor;

pub(super) fn visit_file(visitor: &mut FactVisitor<'_>, file: &File) {
    let guard = cfg_guard(&file.attrs);
    if guard != SyntaxGuard::Ordinary {
        visitor.guard_initial_paths(&guard);
    }
    visitor.with_cfg(&file.attrs, |visitor| {
        visitor.with_local_type_scope(file.items.iter(), |visitor| {
            visitor.with_import_scope(file.items.iter(), |visitor| {
                visit::visit_file(visitor, file);
            });
        });
    });
}

pub(super) fn visit_local(visitor: &mut FactVisitor<'_>, local: &Local) {
    visitor.with_cfg(&local.attrs, |visitor| {
        let input = local
            .init
            .as_ref()
            .map_or(PatternInputMode::Unresolved, |init| {
                visitor.pattern_input_from_expr(&init.expr)
            });
        visitor.with_pattern_input(input, |visitor| visit::visit_local(visitor, local));
        visitor.record_local_bindings(local, input);
    });
}

pub(super) fn visit_struct_pattern(visitor: &mut FactVisitor<'_>, pattern: &PatStruct) {
    visitor.record_struct_pattern(pattern);
    for attribute in &pattern.attrs {
        visitor.visit_attribute(attribute);
    }
    if let Some(qself) = &pattern.qself {
        visitor.visit_qself(qself);
    }
    visitor.with_pattern_type_paths(|visitor| visitor.visit_path(&pattern.path));
    for field in &pattern.fields {
        let input = visitor.pattern_field_input(
            &pattern.path,
            &field.member,
            visitor.current_pattern_input(),
        );
        visitor.with_cfg(&field.attrs, |visitor| {
            for attribute in &field.attrs {
                visitor.visit_attribute(attribute);
            }
            visitor.visit_member(&field.member);
            visitor.with_pattern_input(input, |visitor| visitor.visit_pat(&field.pat));
        });
    }
    if let Some(rest) = &pattern.rest {
        visitor.visit_pat_rest(rest);
    }
}

pub(super) fn visit_reference_pattern(visitor: &mut FactVisitor<'_>, pattern: &PatReference) {
    visitor.with_pattern_input(PatternInputMode::Value, |visitor| {
        visit::visit_pat_reference(visitor, pattern);
    });
}

pub(super) fn visit_typed_pattern(visitor: &mut FactVisitor<'_>, pattern: &PatType) {
    let input = visitor.pattern_input_from_type(&pattern.ty);
    visitor.with_pattern_input(input, |visitor| visit::visit_pat_type(visitor, pattern));
}
