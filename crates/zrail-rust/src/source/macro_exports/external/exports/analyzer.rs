//! External modules and root-exported macro definitions are traversed once.

use std::collections::{BTreeMap, BTreeSet};

use syn::{Item, ItemMacro, ItemMod};

use super::super::archive::VerifiedPackage;
use super::{
    MAX_MODULES, ModuleSurface, PackageExports, conditional, has_attribute, identifier, is_public,
    paths, use_tree,
};

pub(super) fn analyze(package: &VerifiedPackage) -> Result<PackageExports, String> {
    let mut analyzer = Analyzer {
        package,
        exports: PackageExports {
            modules: BTreeMap::new(),
            root_macros: BTreeSet::new(),
            uncertain_root_macros: BTreeSet::new(),
            root_open: None,
        },
        visited: BTreeSet::new(),
    };
    let base = paths::parent(&package.library);
    analyzer.scan_file(&package.library, &[], &base, true)?;
    Ok(analyzer.exports)
}

struct Analyzer<'a> {
    package: &'a VerifiedPackage,
    exports: PackageExports,
    visited: BTreeSet<(String, Vec<String>)>,
}

impl Analyzer<'_> {
    fn scan_file(
        &mut self,
        file: &str,
        module: &[String],
        base: &str,
        public: bool,
    ) -> Result<(), String> {
        if self.visited.len() >= MAX_MODULES {
            return Err("external crate module count exceeds the analysis limit".into());
        }
        if !self.visited.insert((file.to_owned(), module.to_vec())) {
            let reason = "external crate module graph contains a cycle";
            self.unknown_module(module.to_vec(), reason);
            self.mark_open(reason);
            return Ok(());
        }
        let source = self
            .package
            .files
            .get(file)
            .ok_or_else(|| format!("external crate module source {file:?} is unavailable"))?;
        let syntax = syn::parse_file(source)
            .map_err(|error| format!("external crate module {file:?} is invalid: {error}"))?;
        self.scan_items(&syntax.items, module, base, public);
        Ok(())
    }

    fn scan_items(&mut self, items: &[Item], module: &[String], base: &str, public: bool) {
        let mut surface = ModuleSurface {
            public,
            ..ModuleSurface::default()
        };
        for item in items {
            match item {
                Item::Use(item) if is_public(&item.vis) => use_tree::collect(item, &mut surface),
                Item::Macro(item) => self.macro_item(item, module, &mut surface),
                Item::Mod(item) => self.module_item(item, module, base, public),
                _ => {}
            }
        }
        self.insert_surface(module.to_vec(), surface);
    }

    fn macro_item(&mut self, item: &ItemMacro, module: &[String], surface: &mut ModuleSurface) {
        if item.mac.path.is_ident("macro_rules") {
            if has_attribute(&item.attrs, "macro_export") {
                if let Some(name) = &item.ident {
                    let target = if conditional(&item.attrs) {
                        &mut self.exports.uncertain_root_macros
                    } else {
                        &mut self.exports.root_macros
                    };
                    target.insert(identifier(name));
                } else {
                    self.mark_open("external macro definition has no identifier");
                }
            }
        } else {
            let reason = format!(
                "external module {} contains opaque item macro {}!",
                paths::display(module),
                paths::path_text(&item.mac.path)
            );
            surface.open.get_or_insert_with(|| reason.clone());
            self.mark_open(&reason);
        }
    }

    fn module_item(&mut self, item: &ItemMod, module: &[String], base: &str, parent_public: bool) {
        let mut child = module.to_vec();
        let name = identifier(&item.ident);
        child.push(name.clone());
        let public = parent_public && is_public(&item.vis) && !conditional(&item.attrs);
        if conditional(&item.attrs) {
            self.unknown_module(child, "external module declaration is conditional");
            self.mark_open("external crate contains a conditional module");
            return;
        }
        if has_attribute(&item.attrs, "path") {
            self.unknown_module(child, "external module uses an explicit path attribute");
            self.mark_open("external crate contains a path-attributed module");
            return;
        }
        let child_base = paths::join(base, &name);
        if let Some((_, items)) = &item.content {
            self.scan_items(items, &child, &child_base, public);
            return;
        }
        let first = format!("{child_base}.rs");
        let second = paths::join(&child_base, "mod.rs");
        let file = match (
            self.package.files.contains_key(&first),
            self.package.files.contains_key(&second),
        ) {
            (true, false) => Some(first),
            (false, true) => Some(second),
            _ => None,
        };
        if let Some(file) = file {
            if let Err(reason) = self.scan_file(&file, &child, &child_base, public) {
                self.unknown_module(child, &reason);
                self.mark_open(&reason);
            }
        } else {
            let reason = format!(
                "external module source for {} is ambiguous or absent",
                paths::display(&child)
            );
            self.unknown_module(child, &reason);
            self.mark_open(&reason);
        }
    }

    fn unknown_module(&mut self, module: Vec<String>, reason: &str) {
        self.insert_surface(
            module,
            ModuleSurface {
                public: false,
                open: Some(reason.into()),
                ..ModuleSurface::default()
            },
        );
    }

    fn insert_surface(&mut self, module: Vec<String>, surface: ModuleSurface) {
        use std::collections::btree_map::Entry;

        match self.exports.modules.entry(module) {
            Entry::Vacant(entry) => {
                entry.insert(surface);
            }
            Entry::Occupied(mut entry) => {
                let existing = entry.get_mut();
                existing.public = false;
                existing.bindings.clear();
                existing.open = Some("external crate defines a duplicate module path".into());
            }
        }
    }

    fn mark_open(&mut self, reason: &str) {
        self.exports
            .root_open
            .get_or_insert_with(|| reason.to_owned());
    }
}
