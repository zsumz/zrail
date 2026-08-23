//! Conservative file-wide call aliases for scope-sensitive Rust syntax.

use std::collections::BTreeMap;

use syn::{Attribute, Type, UseTree, visit::Visit as _};

use super::{
    SyntaxGuard,
    attributes::is_cfg_test,
    import_helpers::insert_guard,
    visitor_context::{expr_attrs, foreign_attrs, impl_attrs, item_attrs, trait_attrs},
};

#[derive(Default)]
pub(super) struct CallCandidates {
    pub(super) aliases: BTreeMap<String, BTreeMap<String, SyntaxGuard>>,
    pub(super) globs: BTreeMap<String, SyntaxGuard>,
    test_only_context: bool,
}

pub(super) fn collect(file: &syn::File) -> CallCandidates {
    let mut candidates = CallCandidates::default();
    candidates.visit_file(file);
    candidates
}

pub(super) fn normalize(
    candidates: &mut BTreeMap<String, BTreeMap<String, SyntaxGuard>>,
    aliases: &BTreeMap<String, String>,
    alias_guards: &BTreeMap<String, SyntaxGuard>,
) {
    for targets in candidates.values_mut() {
        let mut normalized = BTreeMap::new();
        for (target, guard) in &*targets {
            let (target, alias_guard) = expand_exact_prefix(target, aliases, alias_guards);
            insert_guard(&mut normalized, target, guard.combine(alias_guard));
        }
        *targets = normalized;
    }
}

impl<'ast> syn::visit::Visit<'ast> for CallCandidates {
    fn visit_file(&mut self, file: &'ast syn::File) {
        self.with_cfg(&file.attrs, |visitor| {
            syn::visit::visit_file(visitor, file);
        });
    }

    fn visit_item(&mut self, item: &'ast syn::Item) {
        self.with_cfg(item_attrs(item), |visitor| {
            syn::visit::visit_item(visitor, item);
        });
    }

    fn visit_impl_item(&mut self, item: &'ast syn::ImplItem) {
        self.with_cfg(impl_attrs(item), |visitor| {
            syn::visit::visit_impl_item(visitor, item);
        });
    }

    fn visit_trait_item(&mut self, item: &'ast syn::TraitItem) {
        self.with_cfg(trait_attrs(item), |visitor| {
            syn::visit::visit_trait_item(visitor, item);
        });
    }

    fn visit_foreign_item(&mut self, item: &'ast syn::ForeignItem) {
        self.with_cfg(foreign_attrs(item), |visitor| {
            syn::visit::visit_foreign_item(visitor, item);
        });
    }

    fn visit_expr(&mut self, expression: &'ast syn::Expr) {
        self.with_cfg(expr_attrs(expression), |visitor| {
            syn::visit::visit_expr(visitor, expression);
        });
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        self.with_cfg(&local.attrs, |visitor| {
            syn::visit::visit_local(visitor, local);
        });
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        self.with_cfg(&arm.attrs, |visitor| syn::visit::visit_arm(visitor, arm));
    }

    fn visit_field(&mut self, field: &'ast syn::Field) {
        self.with_cfg(&field.attrs, |visitor| {
            syn::visit::visit_field(visitor, field);
        });
    }

    fn visit_variant(&mut self, variant: &'ast syn::Variant) {
        self.with_cfg(&variant.attrs, |visitor| {
            syn::visit::visit_variant(visitor, variant);
        });
    }

    fn visit_item_extern_crate(&mut self, item: &'ast syn::ItemExternCrate) {
        let alias = item
            .rename
            .as_ref()
            .map_or_else(|| item.ident.to_string(), |(_, name)| name.to_string());
        self.aliases.entry(alias).or_default().insert(
            item.ident.to_string(),
            SyntaxGuard::for_test_only(self.test_only_context),
        );
        syn::visit::visit_item_extern_crate(self, item);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        collect_use(
            self,
            Vec::new(),
            &item.tree,
            SyntaxGuard::for_test_only(self.test_only_context),
        );
        syn::visit::visit_item_use(self, item);
    }

    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        if let Type::Path(target) = item.ty.as_ref()
            && target.qself.is_none()
        {
            self.aliases
                .entry(item.ident.to_string())
                .or_default()
                .insert(
                    path_text(&target.path),
                    SyntaxGuard::for_test_only(self.test_only_context),
                );
        }
        syn::visit::visit_item_type(self, item);
    }
}

impl CallCandidates {
    fn with_cfg(&mut self, attributes: &[Attribute], visit: impl FnOnce(&mut Self)) {
        let previous = self.test_only_context;
        self.test_only_context |= attributes.iter().any(is_cfg_test);
        visit(self);
        self.test_only_context = previous;
    }
}

fn collect_use(
    candidates: &mut CallCandidates,
    prefix: Vec<String>,
    tree: &UseTree,
    guard: SyntaxGuard,
) {
    match tree {
        UseTree::Path(path) => {
            let mut nested = prefix;
            nested.push(path.ident.to_string());
            collect_use(candidates, nested, &path.tree, guard);
        }
        UseTree::Name(name) if name.ident == "self" => {
            if let Some(alias) = prefix.last() {
                insert_alias(candidates, alias, &prefix, guard);
            }
        }
        UseTree::Name(name) => {
            let mut target = prefix;
            target.push(name.ident.to_string());
            insert_alias(candidates, &name.ident.to_string(), &target, guard);
        }
        UseTree::Rename(rename) => {
            let mut target = prefix;
            if rename.ident != "self" {
                target.push(rename.ident.to_string());
            }
            insert_alias(candidates, &rename.rename.to_string(), &target, guard);
        }
        UseTree::Glob(_) => insert_guard(&mut candidates.globs, prefix.join("::"), guard),
        UseTree::Group(group) => {
            for item in &group.items {
                collect_use(candidates, prefix.clone(), item, guard);
            }
        }
    }
}

fn insert_alias(
    candidates: &mut CallCandidates,
    alias: &str,
    target: &[String],
    guard: SyntaxGuard,
) {
    if !target.is_empty() {
        candidates
            .aliases
            .entry(alias.to_owned())
            .or_default()
            .insert(target.join("::"), guard);
    }
}

fn expand_exact_prefix(
    target: &str,
    aliases: &BTreeMap<String, String>,
    alias_guards: &BTreeMap<String, SyntaxGuard>,
) -> (String, SyntaxGuard) {
    let mut segments = target.split("::");
    let first = segments.next().unwrap_or_default();
    let remainder = segments.collect::<Vec<_>>();
    aliases.get(first).map_or_else(
        || (target.to_owned(), SyntaxGuard::Ordinary),
        |prefix| {
            let path = if remainder.is_empty() {
                prefix.clone()
            } else {
                format!("{prefix}::{}", remainder.join("::"))
            };
            (path, alias_guards.get(first).copied().unwrap_or_default())
        },
    )
}

fn path_text(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}
