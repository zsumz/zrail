//! Exact frozen transport collector; executed only in trusted qualification tests.

#[path = "legacy_api.rs"]
mod api;
pub(super) use api::{
    associated, check_associated, check_detector_associated, check_detector_owners, check_owners,
    expected, expected_associated, observed, owners, repository_associated,
};

use super::selection::{display_path, is_test, read};
use std::collections::{BTreeMap, BTreeSet};
use syn::{ExprMethodCall, ExprPath, File, ItemImpl, ItemUse, Path, Type, UseTree, visit::Visit};

const ASSOCIATED_CALLS: [&str; 13] = [
    "ConnectionSet::new",
    "ConnectionSet::turn_component",
    "ConnectionSet::poll_io",
    "ConnectionSet::wake_handle",
    "ConnectionSet::pulse_handle",
    "DirectSet::new",
    "DirectSet::turn_component",
    "DirectSet::poll_io",
    "DirectSet::wake_handle",
    "DirectSet::pulse_handle",
    "Source::register",
    "Source::reregister",
    "Source::deregister",
];
const GUARDED_RENAMES: [&str; 5] = [
    "ConnectionSet",
    "DirectSet",
    "RegisteredTransport",
    "SlotTransport",
    "Source",
];
const SELECTOR_METHODS: [&str; 4] = ["turn_component", "poll_io", "wake_handle", "pulse_handle"];

#[derive(Debug, Default, Eq, PartialEq)]
struct AuthorityInventory {
    connection_set_files: BTreeSet<String>,
    associated_calls: BTreeMap<String, usize>,
    renamed_authorities: BTreeSet<String>,
    selector_methods: BTreeMap<String, usize>,
    transport_impls: BTreeSet<String>,
}

fn source_inventory(path: &str, source: &str) -> AuthorityInventory {
    let syntax = syn::parse_file(source)
        .unwrap_or_else(|error| panic!("parse adversarial authority source: {error}"));
    let mut inventory = AuthorityInventory::default();
    inspect(&syntax, path, &mut inventory);
    inventory
}

fn inspect(syntax: &File, path: &str, inventory: &mut AuthorityInventory) {
    AuthorityVisitor { path, inventory }.visit_file(syntax);
}

struct AuthorityVisitor<'a> {
    path: &'a str,
    inventory: &'a mut AuthorityInventory,
}

impl<'ast> Visit<'ast> for AuthorityVisitor<'_> {
    fn visit_path(&mut self, path: &'ast Path) {
        if path
            .segments
            .iter()
            .any(|segment| segment.ident == "ConnectionSet")
        {
            self.inventory.connection_set_files.insert(self.path.into());
        }
        syn::visit::visit_path(self, path);
    }

    fn visit_expr_path(&mut self, path: &'ast ExprPath) {
        if let Some(authority) = associated_authority(&path.path) {
            increment(
                &mut self.inventory.associated_calls,
                format!("{}:{authority}", self.path),
            );
        }
        syn::visit::visit_expr_path(self, path);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        let method = call.method.to_string();
        if SELECTOR_METHODS.contains(&method.as_str()) {
            increment(
                &mut self.inventory.selector_methods,
                format!("{}:{method}", self.path),
            );
        }
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        if let Some((_, trait_path, _)) = &item.trait_
            && let Some(trait_name) = trait_path.segments.last()
            && let Some(type_name) = type_name(&item.self_ty)
            && (matches!(
                trait_name.ident.to_string().as_str(),
                "RegisteredTransport" | "SlotTransport"
            ) || (trait_name.ident == "Source" && type_name == "DirectRustlsTransport"))
        {
            self.inventory
                .transport_impls
                .insert(format!("{}:{type_name}:{}", self.path, trait_name.ident));
        }
        syn::visit::visit_item_impl(self, item);
    }

    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        record_guarded_renames(
            &item.tree,
            self.path,
            &mut self.inventory.renamed_authorities,
        );
        syn::visit::visit_item_use(self, item);
    }
}

fn record_guarded_renames(tree: &UseTree, path: &str, renames: &mut BTreeSet<String>) {
    match tree {
        UseTree::Path(tree) => record_guarded_renames(&tree.tree, path, renames),
        UseTree::Group(group) => {
            for tree in &group.items {
                record_guarded_renames(tree, path, renames);
            }
        }
        UseTree::Rename(rename) => {
            let authority = rename.ident.to_string();
            if GUARDED_RENAMES.contains(&authority.as_str()) {
                renames.insert(format!("{path}:{authority} as {}", rename.rename));
            }
        }
        UseTree::Name(_) | UseTree::Glob(_) => {}
    }
}

fn associated_authority(path: &Path) -> Option<String> {
    let segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    let [.., owner, method] = segments.as_slice() else {
        return None;
    };
    let authority = format!("{owner}::{method}");
    ASSOCIATED_CALLS
        .contains(&authority.as_str())
        .then_some(authority)
}

fn type_name(ty: &Type) -> Option<String> {
    let Type::Path(path) = ty else {
        return None;
    };
    path.path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
}

fn increment(counts: &mut BTreeMap<String, usize>, key: String) {
    *counts.entry(key).or_default() += 1;
}

fn counts(entries: &[(&str, usize)]) -> BTreeMap<String, usize> {
    entries
        .iter()
        .map(|&(key, value)| (key.into(), value))
        .collect()
}

fn expected_selector_methods() -> BTreeMap<String, usize> {
    counts(&[
        ("src/reactor/backend.rs:wake_handle", 2),
        ("src/reactor/backend.rs:pulse_handle", 2),
        ("src/reactor/host.rs:wake_handle", 1),
        ("src/reactor/host/construction.rs:wake_handle", 1),
        ("src/reactor/host/construction.rs:pulse_handle", 2),
        ("src/reactor/direct_plaintext/backend.rs:wake_handle", 3),
        ("src/reactor/direct_plaintext/backend.rs:pulse_handle", 3),
        ("src/reactor/direct_plaintext/runtime.rs:wake_handle", 1),
        ("src/reactor/direct_plaintext/runtime.rs:pulse_handle", 1),
        (
            "src/reactor/direct_plaintext/cluster_runtime/backend.rs:wake_handle",
            2,
        ),
        (
            "src/reactor/direct_plaintext/cluster_runtime/backend.rs:pulse_handle",
            2,
        ),
    ])
}

fn repository_inventory(root: &std::path::Path, roots: &[String]) -> AuthorityInventory {
    let workspace_root = || root.to_path_buf();
    let rust_files = |root: &std::path::Path| crate::rules::legacy_driver_paths(root, roots);
    let root = workspace_root();
    let mut inventory = AuthorityInventory::default();
    for path in rust_files(&root) {
        if is_test(&root, &path) {
            continue;
        }
        let relative = display_path(&root, &path);
        let source = read(&path);
        let syntax =
            syn::parse_file(&source).unwrap_or_else(|error| panic!("parse {relative}: {error}"));
        inspect(&syntax, &relative, &mut inventory);
    }
    inventory
}

pub(super) fn repository(root: &std::path::Path, roots: &[String]) -> BTreeMap<String, usize> {
    repository_inventory(root, roots).selector_methods
}

pub(super) fn check_selector_methods(selector_methods: BTreeMap<String, usize>) {
    let actual = AuthorityInventory {
        selector_methods,
        ..AuthorityInventory::default()
    };
    assert_eq!(actual.selector_methods, expected_selector_methods());
}

pub(super) fn check_detector_methods(selector_methods: BTreeMap<String, usize>) {
    let actual = AuthorityInventory {
        selector_methods,
        ..AuthorityInventory::default()
    };
    assert_eq!(
        actual.selector_methods,
        counts(&[("src/reactor/rogue.rs:poll_io", 1)])
    );
}
