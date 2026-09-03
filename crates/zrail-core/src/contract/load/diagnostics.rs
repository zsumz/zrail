//! Contract parse diagnostics add context for structurally ambiguous aliases.

pub(super) fn parse_error(error: &toml::de::Error) -> String {
    let detail = error.to_string();
    if detail.contains("duplicate field `resolution`") {
        format!(
            "{detail}\nhelp: `resolution` is canonical and `binding` is its legacy alias; do not set both spellings in one macro authority entry"
        )
    } else {
        detail
    }
}

#[cfg(test)]
#[path = "diagnostics_test.rs"]
mod diagnostics_test;
