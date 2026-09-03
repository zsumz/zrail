//! Macro allowance identity is the authored name plus exact provenance.

pub(super) fn of(allowance: &crate::MacroExpansionAllow) -> (String, String) {
    let provenance = if let Some(definition) = &allowance.definition {
        format!("definition:{definition}")
    } else if let Some(source) = &allowance.source {
        format!("source:{}", source.identity())
    } else {
        "unbound".into()
    };
    (allowance.name.clone(), provenance)
}

#[cfg(test)]
#[path = "identity_test.rs"]
mod identity_test;
