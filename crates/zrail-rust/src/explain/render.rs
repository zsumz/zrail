//! Human and JSON renderings share the same effective path-policy facts.

use super::{PathExplanation, owners};

impl PathExplanation {
    pub fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self).map(|mut value| {
            value.push('\n');
            value
        })
    }

    pub fn human(&self) -> String {
        format!(
            concat!(
                "path: {}\nclass: {}\nreachability: {}\npackage: {}\nlayer: {}\n",
                "profiles: {}\nscopes: {}\ndependency layers: {}\n",
                "external dependencies: {}\ndenied effects: {}\ndenied symbols: {}\n",
                "denied methods: {}\ndenied macros: {}\nmacro expansion: {}\n",
                "allowed macro expansions: {}\nopaque macro inputs: {}\n",
                "content-bound macro definitions: {}\nunsafe code: {}\n",
                "lint suppressions: {}\nexpected sibling test: {}\ninvariants: {}\n",
                "capability owners: {}\ncall owners: {}\nbudget: target {}, hard {}\n",
                "declarative shape: {}\nmodule docs: {}\nsibling tests: {}\n",
            ),
            self.path,
            self.file_class,
            self.reachability,
            self.package.as_deref().unwrap_or("<none>"),
            self.layer.as_deref().unwrap_or("<none>"),
            display_list(&self.profiles),
            display_list(&self.scopes),
            display_list(&self.permitted_dependency_layers),
            self.external_dependencies.as_deref().unwrap_or("<none>"),
            display_list(&self.denied_effects),
            display_list(&self.denied_symbols),
            display_list(&self.denied_methods),
            display_list(&self.denied_macros),
            self.macro_expansion,
            display_list(&self.allowed_macro_expansions),
            display_list(&self.opaque_macro_inputs),
            display_list(&self.content_bound_macro_definitions),
            self.unsafe_code,
            self.lint_suppressions,
            self.expected_sibling_test.as_deref().unwrap_or("<none>"),
            display_list(&self.invariants),
            owners::display(&self.capability_owners),
            owners::display_calls(&self.call_owners),
            display_optional_number(self.design_target),
            display_optional_number(self.hard_ceiling),
            display_optional_bool(self.declarative_shape),
            self.module_docs_required,
            self.sibling_tests_required
        )
    }
}

fn display_list(values: &[String]) -> String {
    if values.is_empty() {
        "<none>".into()
    } else {
        values.join(", ")
    }
}

const fn display_optional_bool(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "<not applicable>",
    }
}

fn display_optional_number(value: Option<usize>) -> String {
    value.map_or_else(|| "<not enforced>".into(), |value| value.to_string())
}
