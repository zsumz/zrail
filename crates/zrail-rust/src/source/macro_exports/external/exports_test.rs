//! External module export surfaces retain per-name certainty.

use std::collections::BTreeMap;

use super::{PackageExports, VerifiedPackage};

#[test]
fn direct_prelude_names_are_present_uncertain_or_absent_per_name() {
    let package = package(
        concat!(
            "#[macro_export]\nmacro_rules! reviewed { () => {} }\n",
            "#[macro_export]\nmacro_rules! hidden { () => {} }\n",
            "pub mod prelude",
            ";\n",
        ),
        "pub use crate::reviewed;\npub use crate::ordinary;\n",
    );
    let exports = PackageExports::analyze(&package)
        .expect("analyze package")
        .module(&["prelude".into()]);

    assert!(exports.macros.contains("reviewed"));
    assert!(exports.uncertain.contains_key("ordinary"));
    assert!(!exports.macros.contains("hidden"));
    assert!(!exports.uncertain.contains_key("hidden"));
    assert!(exports.open.is_none());
}

#[test]
fn conditional_and_glob_reexports_remain_unknown() {
    let package = package(
        concat!(
            "#[macro_export]\nmacro_rules! reviewed { () => {} }\n",
            "pub mod prelude",
            ";\n",
        ),
        "#[cfg(feature = \"macros\")]\npub use crate::reviewed;\npub use crate::other::*;\n",
    );
    let exports = PackageExports::analyze(&package)
        .expect("analyze package")
        .module(&["prelude".into()]);

    assert!(!exports.macros.contains("reviewed"));
    assert!(
        exports
            .uncertain
            .get("reviewed")
            .is_some_and(|reason| reason.contains("conditional"))
    );
    assert!(
        exports
            .open
            .as_deref()
            .is_some_and(|reason| reason.contains("glob"))
    );
}

#[test]
fn public_glob_overrides_an_exact_named_reexport() {
    let package = package(
        concat!(
            "#[macro_export]\nmacro_rules! reviewed { () => {} }\n",
            "pub mod prelude",
            ";\n",
        ),
        "pub use crate::reviewed;\npub use crate::other::*;\n",
    );
    let exports = PackageExports::analyze(&package)
        .expect("analyze package")
        .module(&["prelude".into()]);

    assert!(!exports.macros.contains("reviewed"));
    assert!(exports.open.is_some());
}

#[test]
fn duplicate_named_reexports_are_uncertain() {
    let package = package(
        concat!(
            "#[macro_export]\nmacro_rules! reviewed { () => {} }\n",
            "pub mod prelude",
            ";\n",
        ),
        "pub use crate::reviewed;\npub use crate::reviewed as reviewed;\n",
    );
    let exports = PackageExports::analyze(&package)
        .expect("analyze package")
        .module(&["prelude".into()]);

    assert!(!exports.macros.contains("reviewed"));
    assert!(
        exports
            .uncertain
            .get("reviewed")
            .is_some_and(|reason| reason.contains("duplicate"))
    );
}

#[test]
fn raw_macro_identifiers_use_their_canonical_spelling() {
    let package = package(
        concat!(
            "#[macro_export]\nmacro_rules! r#type { () => {} }\n",
            "pub mod prelude",
            ";\n",
        ),
        "pub use crate::r#type;\n",
    );
    let exports = PackageExports::analyze(&package)
        .expect("analyze package")
        .module(&["prelude".into()]);

    assert!(exports.macros.contains("type"));
    assert!(!exports.macros.contains("r#type"));
}

#[test]
fn duplicate_module_paths_do_not_retain_an_exact_surface() {
    let package = VerifiedPackage {
        files: BTreeMap::from([(
            "src/lib.rs".into(),
            concat!(
                "#[macro_export]\nmacro_rules! reviewed { () => {} }\n",
                "pub mod prelude { pub use crate::reviewed; }\n",
                "pub mod prelude { pub use crate::reviewed; }\n",
            )
            .into(),
        )]),
        library: "src/lib.rs".into(),
    };
    let exports = PackageExports::analyze(&package)
        .expect("analyze package")
        .module(&["prelude".into()]);

    assert!(exports.macros.is_empty());
    assert!(exports.open.is_some());
}

fn package(library: &str, prelude: &str) -> VerifiedPackage {
    VerifiedPackage {
        files: BTreeMap::from([
            ("src/lib.rs".into(), library.into()),
            ("src/prelude.rs".into(), prelude.into()),
        ]),
        library: "src/lib.rs".into(),
    }
}
