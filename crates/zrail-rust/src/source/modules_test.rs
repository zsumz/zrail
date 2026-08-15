//! Module facts preserve explicit paths, inline bases, and conditional uncertainty.

use super::module_declarations;

#[test]
fn external_modules_retain_inline_path_context() {
    let source = format!(
        r#"
        mod {direct};
        #[path = "renamed.rs"] mod {renamed};
        #[path = "thread_files"] mod thread {{
            #[path = "tls.rs"] mod {local_data};
        }}
        "#,
        direct = "direct",
        renamed = "renamed",
        local_data = "local_data",
    );
    let syntax = syn::parse_file(&source).expect("parse modules");

    let declarations = module_declarations(&syntax);

    assert_eq!(declarations[0].name, "direct");
    assert_eq!(declarations[1].path.as_deref(), Some("renamed.rs"));
    assert_eq!(declarations[2].name, "local_data");
    assert_eq!(declarations[2].path.as_deref(), Some("tls.rs"));
    assert_eq!(
        declarations[2].inline_ancestors[0].path.as_deref(),
        Some("thread_files")
    );
}

#[test]
fn conditional_path_attributes_are_unresolved() {
    let source = format!(
        r#"#[cfg_attr(unix, path = "unix.rs")] mod {platform};"#,
        platform = "platform"
    );
    let syntax = syn::parse_file(&source).expect("parse conditional path");

    let declarations = module_declarations(&syntax);

    assert!(declarations[0].unresolved_path);
}

#[test]
fn duplicate_path_attributes_are_unresolved() {
    let source = format!(
        r#"#[path = "one.rs"] #[path = "two.rs"] mod {module};"#,
        module = "duplicate"
    );
    let syntax = syn::parse_file(&source).expect("parse duplicate paths");

    let declarations = module_declarations(&syntax);

    assert!(declarations[0].unresolved_path);
}
