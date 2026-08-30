//! One declaration representation is resolved independently in each Cargo domain.

use zrail_core::RustTypeContract;

use crate::source::{
    CompilationDomain, GuardAvailability, RustFileFacts, TypeDeclarationFact, TypeDeclarationKind,
};

use super::super::{RuleContext, identity};

pub(crate) struct ResolvedDeclarationShape {
    pub(crate) domain: CompilationDomain,
    pub(crate) occurrence: Option<usize>,
    pub(crate) kind: TypeDeclarationKind,
    pub(crate) visibility: String,
    pub(crate) leaf_module: Result<bool, String>,
    pub(crate) fields: Result<Option<Vec<ResolvedFieldShape>>, String>,
    pub(crate) opacity: Option<String>,
}

pub(crate) struct ResolvedFieldShape {
    pub(crate) name: String,
    pub(crate) visibility: String,
    pub(crate) type_identity: String,
}

impl ResolvedDeclarationShape {
    pub(crate) fn is_exact(&self) -> bool {
        self.opacity.is_none() && self.leaf_module.is_ok() && self.fields.is_ok()
    }
}

pub(crate) fn resolve(
    context: &RuleContext<'_>,
    policy: &RustTypeContract,
    file: &RustFileFacts,
    declaration: &TypeDeclarationFact,
) -> Vec<ResolvedDeclarationShape> {
    context
        .compilation_domains
        .get(&file.relative)
        .into_iter()
        .flatten()
        .filter(|domain| {
            identity::within_reachability(domain, policy.reachability)
                && declaration
                    .guard
                    .availability_in_domain(domain)
                    .is_available()
        })
        .flat_map(|domain| {
            let occurrences = declaration
                .module_occurrences
                .iter()
                .filter(|occurrence| {
                    &occurrence.domain == domain
                        && occurrence.identity.as_ref().is_none_or(|name| {
                            identity::normalize(name) == identity::normalize(&policy.identity)
                        })
                })
                .map(|occurrence| (Some(occurrence.instance.0), occurrence.leaf.clone()))
                .collect::<Vec<_>>();
            let occurrences = if occurrences.is_empty() {
                vec![(
                    None,
                    Err("logical module occurrences are unresolved".into()),
                )]
            } else {
                occurrences
            };
            occurrences
                .into_iter()
                .map(|(occurrence, leaf_module)| ResolvedDeclarationShape {
                    domain: domain.clone(),
                    occurrence,
                    kind: declaration.kind,
                    visibility: declaration.visibility.clone(),
                    leaf_module,
                    fields: fields(file, declaration, domain),
                    opacity: opacity(declaration, domain),
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn opacity(declaration: &TypeDeclarationFact, domain: &CompilationDomain) -> Option<String> {
    if declaration.replacing_mounts.contains(domain) {
        return Some("an item-replacing attribute on an ancestor module mount prevents exact declaration shape".into());
    }
    if let Some(occurrence) = declaration.replacement_macros.iter().find(|occurrence| {
        occurrence
            .guard
            .availability_in_domain(domain)
            .is_available()
    }) {
        return Some(format!(
            "item-replacing attribute at {:?} cannot preserve exact declaration shape; namespace authority is not shape authority",
            occurrence.span
        ));
    }
    (declaration.guard.availability_in_domain(domain) == GuardAvailability::Possible)
        .then(|| "declaration availability is unresolved".into())
}

fn fields(
    file: &RustFileFacts,
    declaration: &TypeDeclarationFact,
    domain: &CompilationDomain,
) -> Result<Option<Vec<ResolvedFieldShape>>, String> {
    let Some(fields) = &declaration.fields else {
        return Ok(None);
    };
    let mut resolved = Vec::new();
    for field in fields {
        match field.guard.availability_in_domain(domain) {
            GuardAvailability::Absent => continue,
            GuardAvailability::Possible => {
                return Err(format!(
                    "field {:?} availability is unresolved ({})",
                    field.name,
                    field.guard.canonical_name()
                ));
            }
            GuardAvailability::Exact => {}
        }
        resolved.push(ResolvedFieldShape {
            name: field.name.clone(),
            visibility: field.visibility.clone(),
            type_identity: super::render::render_source(&field.type_shape, file, domain)
                .map_err(|error| format!("field {:?} type is unresolved: {error}", field.name))?,
        });
    }
    Ok(Some(resolved))
}
