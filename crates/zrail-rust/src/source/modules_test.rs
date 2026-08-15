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

#[test]
fn external_modules_inherit_nested_inline_test_context() {
    let source = format!(
        r"
        #[cfg(test)]
        mod tests {{
            mod {support};
            mod outer {{
                mod inner {{
                    mod {nested};
                }}
            }}
        }}
    ",
        support = "support",
        nested = "nested",
    );
    let syntax = syn::parse_file(&source).expect("parse nested test modules");

    let declarations = module_declarations(&syntax);

    assert_eq!(declarations.len(), 2);
    assert!(declarations.iter().all(|declaration| declaration.cfg_test));
    assert_eq!(declarations[0].name, "support");
    assert_eq!(declarations[1].name, "nested");
}

#[test]
fn external_modules_inherit_file_function_and_method_test_context() {
    let source = r"
        struct Harness;

        #[cfg(test)]
        fn function() { mod function_support; }

        impl Harness {
            #[cfg(test)]
            fn method() { mod method_support; }
        }
    ";
    let syntax = syn::parse_file(source).expect("parse local test modules");

    let declarations = module_declarations(&syntax);

    assert_eq!(declarations.len(), 2);
    assert!(declarations.iter().all(|declaration| declaration.cfg_test));
}

#[test]
fn external_modules_inherit_file_inner_test_context() {
    let syntax =
        syn::parse_file("#![cfg(test)]\nmod file_support;\n").expect("parse file test module");

    let declarations = module_declarations(&syntax);

    assert_eq!(declarations.len(), 1);
    assert!(declarations[0].cfg_test);
}
