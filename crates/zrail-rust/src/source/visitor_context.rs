//! Effective test-only cfg context follows every attribute-bearing syntax node.

use syn::{Attribute, Expr, ForeignItem, ImplItem, Item, TraitItem};

use super::{SyntaxGuard, attributes::is_cfg_test, visitor::FactVisitor};

impl FactVisitor<'_> {
    pub(super) const fn syntax_guard(&self) -> SyntaxGuard {
        SyntaxGuard::for_test_only(self.test_only_context)
    }

    pub(super) fn with_cfg(&mut self, attributes: &[Attribute], visit: impl FnOnce(&mut Self)) {
        let previous = self.test_only_context;
        let guarded = attributes.iter().any(is_cfg_test);
        let checkpoint = (!previous && guarded).then(|| FactCheckpoint::capture(self));
        self.test_only_context |= guarded;
        visit(self);
        if let Some(checkpoint) = checkpoint {
            checkpoint.mark_test_only(self);
        }
        self.test_only_context = previous;
    }

    pub(super) fn with_lexical_scope(
        &mut self,
        span: proc_macro2::Span,
        visit: impl FnOnce(&mut Self),
    ) {
        self.lexical_scope.push(super::fact::source_span(span));
        visit(self);
        self.lexical_scope.pop();
    }

    pub(super) fn guard_initial_paths(&mut self) {
        for path in &mut self.paths {
            path.mark_test_only();
        }
    }
}

#[derive(Clone, Copy)]
struct FactCheckpoint {
    paths: usize,
    calls: usize,
    methods: usize,
    macros: usize,
    expansions: usize,
    opaque_inputs: usize,
    compile_effects: usize,
    lint_suppressions: usize,
    unsafe_constructs: usize,
    tests: usize,
    item_macros: usize,
    opaque_binding_macros: usize,
    macro_definitions: usize,
}

impl FactCheckpoint {
    fn capture(visitor: &FactVisitor<'_>) -> Self {
        Self {
            paths: visitor.paths.len(),
            calls: visitor.calls.len(),
            methods: visitor.methods.len(),
            macros: visitor.macros.len(),
            expansions: visitor.macro_expansions.len(),
            opaque_inputs: visitor.opaque_macro_inputs.len(),
            compile_effects: visitor.compile_effects.len(),
            lint_suppressions: visitor.lint_suppressions.len(),
            unsafe_constructs: visitor.unsafe_constructs.len(),
            tests: visitor.tests.len(),
            item_macros: visitor.item_macros.len(),
            opaque_binding_macros: visitor.opaque_binding_macros.len(),
            macro_definitions: visitor.macro_definitions.len(),
        }
    }

    fn mark_test_only(self, visitor: &mut FactVisitor<'_>) {
        mark(&mut visitor.paths[self.paths..]);
        mark(&mut visitor.calls[self.calls..]);
        mark(&mut visitor.methods[self.methods..]);
        mark(&mut visitor.macros[self.macros..]);
        mark(&mut visitor.lint_suppressions[self.lint_suppressions..]);
        mark(&mut visitor.unsafe_constructs[self.unsafe_constructs..]);
        mark(&mut visitor.tests[self.tests..]);
        mark(&mut visitor.item_macros[self.item_macros..]);
        mark(&mut visitor.opaque_binding_macros[self.opaque_binding_macros..]);
        for definition in &mut visitor.macro_definitions[self.macro_definitions..] {
            definition.mark_test_only();
        }
        for expansion in &mut visitor.macro_expansions[self.expansions..] {
            expansion.mark_test_only();
        }
        for expansion in &mut visitor.opaque_macro_inputs[self.opaque_inputs..] {
            expansion.mark_test_only();
        }
        for effect in &mut visitor.compile_effects[self.compile_effects..] {
            effect.invocation.mark_test_only();
        }
    }
}

fn mark(facts: &mut [super::ObservedFact]) {
    for fact in facts {
        fact.mark_test_only();
    }
}

pub(super) fn item_attrs(item: &Item) -> &[Attribute] {
    match item {
        Item::Const(value) => &value.attrs,
        Item::Enum(value) => &value.attrs,
        Item::ExternCrate(value) => &value.attrs,
        Item::Fn(value) => &value.attrs,
        Item::ForeignMod(value) => &value.attrs,
        Item::Impl(value) => &value.attrs,
        Item::Macro(value) => &value.attrs,
        Item::Mod(value) => &value.attrs,
        Item::Static(value) => &value.attrs,
        Item::Struct(value) => &value.attrs,
        Item::Trait(value) => &value.attrs,
        Item::TraitAlias(value) => &value.attrs,
        Item::Type(value) => &value.attrs,
        Item::Union(value) => &value.attrs,
        Item::Use(value) => &value.attrs,
        _ => &[],
    }
}

pub(super) fn impl_attrs(item: &ImplItem) -> &[Attribute] {
    match item {
        ImplItem::Const(value) => &value.attrs,
        ImplItem::Fn(value) => &value.attrs,
        ImplItem::Type(value) => &value.attrs,
        ImplItem::Macro(value) => &value.attrs,
        _ => &[],
    }
}

pub(super) fn trait_attrs(item: &TraitItem) -> &[Attribute] {
    match item {
        TraitItem::Const(value) => &value.attrs,
        TraitItem::Fn(value) => &value.attrs,
        TraitItem::Type(value) => &value.attrs,
        TraitItem::Macro(value) => &value.attrs,
        _ => &[],
    }
}

pub(super) fn foreign_attrs(item: &ForeignItem) -> &[Attribute] {
    match item {
        ForeignItem::Fn(value) => &value.attrs,
        ForeignItem::Static(value) => &value.attrs,
        ForeignItem::Type(value) => &value.attrs,
        ForeignItem::Macro(value) => &value.attrs,
        _ => &[],
    }
}

pub(super) fn expr_attrs(expression: &Expr) -> &[Attribute] {
    match expression {
        Expr::Array(value) => &value.attrs,
        Expr::Assign(value) => &value.attrs,
        Expr::Async(value) => &value.attrs,
        Expr::Await(value) => &value.attrs,
        Expr::Binary(value) => &value.attrs,
        Expr::Block(value) => &value.attrs,
        Expr::Break(value) => &value.attrs,
        Expr::Call(value) => &value.attrs,
        Expr::Cast(value) => &value.attrs,
        Expr::Closure(value) => &value.attrs,
        Expr::Const(value) => &value.attrs,
        Expr::Continue(value) => &value.attrs,
        Expr::Field(value) => &value.attrs,
        Expr::ForLoop(value) => &value.attrs,
        Expr::Group(value) => &value.attrs,
        Expr::If(value) => &value.attrs,
        Expr::Index(value) => &value.attrs,
        Expr::Infer(value) => &value.attrs,
        Expr::Let(value) => &value.attrs,
        Expr::Lit(value) => &value.attrs,
        Expr::Loop(value) => &value.attrs,
        Expr::Macro(value) => &value.attrs,
        Expr::Match(value) => &value.attrs,
        Expr::MethodCall(value) => &value.attrs,
        Expr::Paren(value) => &value.attrs,
        Expr::Path(value) => &value.attrs,
        Expr::Range(value) => &value.attrs,
        Expr::RawAddr(value) => &value.attrs,
        Expr::Reference(value) => &value.attrs,
        Expr::Repeat(value) => &value.attrs,
        Expr::Return(value) => &value.attrs,
        Expr::Struct(value) => &value.attrs,
        Expr::Try(value) => &value.attrs,
        Expr::TryBlock(value) => &value.attrs,
        Expr::Tuple(value) => &value.attrs,
        Expr::Unary(value) => &value.attrs,
        Expr::Unsafe(value) => &value.attrs,
        Expr::While(value) => &value.attrs,
        Expr::Yield(value) => &value.attrs,
        _ => &[],
    }
}

#[cfg(test)]
#[path = "visitor_context_test.rs"]
mod visitor_context_test;
