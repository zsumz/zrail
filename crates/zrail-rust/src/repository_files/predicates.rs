//! Closed physical path, name, raw-text, and byte-equality predicates.

use std::{collections::BTreeSet, path::Path};

use zrail_core::{
    RepositoryCaseMode, RepositoryEntryMode, RepositoryFilePredicate, RepositoryNameBasis,
    RepositoryNamePart,
};

use super::{
    boundary, documents, input::Inputs, literal, model::GovernedRepositoryFile, text_structure,
};

pub(super) fn evaluate(
    root: &Path,
    observed: &mut GovernedRepositoryFile,
    inputs: &mut Inputs,
) -> Result<&'static str, String> {
    let (satisfied, diagnostic) = match &observed.policy.predicate {
        RepositoryFilePredicate::Count { minimum, maximum } => {
            let count = observed.entries.len();
            (
                count >= *minimum && maximum.is_none_or(|max| count <= max),
                "REP-FILE-001",
            )
        }
        RepositoryFilePredicate::ExactPaths { paths } => {
            let expected = paths.iter().map(String::as_str).collect::<BTreeSet<_>>();
            for entry in &mut observed.entries {
                entry.satisfied = expected.contains(entry.path.as_str());
            }
            (
                expected
                    == observed
                        .entries
                        .iter()
                        .map(|entry| entry.path.as_str())
                        .collect(),
                "REP-FILE-002",
            )
        }
        RepositoryFilePredicate::ForbiddenNames {
            names,
            part,
            basis,
            case,
        } => {
            for entry in &mut observed.entries {
                let path = match basis {
                    RepositoryNameBasis::Repository => entry.path.clone(),
                    RepositoryNameBasis::Filesystem => root
                        .join(&entry.path)
                        .to_str()
                        .ok_or("checkout path is not UTF-8")?
                        .into(),
                };
                entry.forbidden_names = forbidden_names(&path, names, *part, *case);
                entry.satisfied = entry.forbidden_names.is_empty();
            }
            (true, "REP-FILE-003")
        }
        RepositoryFilePredicate::Literal(policy) => {
            for entry in &mut observed.entries {
                let bytes = inputs.read(root, entry)?;
                literal::evaluate(policy, &bytes, entry)?;
            }
            (
                !literal::requires_file(policy) || !observed.entries.is_empty(),
                "REP-FILE-004",
            )
        }
        predicate @ (RepositoryFilePredicate::LiteralOrder { .. }
        | RepositoryFilePredicate::LiteralBetween { .. }
        | RepositoryFilePredicate::LineValuesAllowed { .. }) => {
            for entry in &mut observed.entries {
                let bytes = inputs.read(root, entry)?;
                text_structure::evaluate(predicate, &bytes, entry)?;
            }
            (
                matches!(predicate, RepositoryFilePredicate::LineValuesAllowed { .. })
                    || !observed.entries.is_empty(),
                "REP-FILE-008",
            )
        }
        RepositoryFilePredicate::BytesEqual { other, utf8 } => {
            observed.reference = boundary::probe(root, other)?
                .map(|entry| boundary::observe(root, &entry, RepositoryEntryMode::File))
                .transpose()?
                .flatten();
            let reference = observed
                .reference
                .as_mut()
                .map(|entry| {
                    let bytes = inputs.read(root, entry)?;
                    if *utf8 {
                        entry.valid_utf8 = Some(std::str::from_utf8(&bytes).is_ok());
                        entry.satisfied = entry.valid_utf8 == Some(true);
                    }
                    Ok::<_, String>(bytes)
                })
                .transpose()?;
            for entry in &mut observed.entries {
                let bytes = inputs.read(root, entry)?;
                if *utf8 {
                    entry.valid_utf8 = Some(std::str::from_utf8(&bytes).is_ok());
                }
                entry.satisfied = reference
                    .as_ref()
                    .is_some_and(|expected| **expected == *bytes)
                    && entry.valid_utf8 != Some(false);
            }
            (
                observed
                    .reference
                    .as_ref()
                    .is_some_and(|entry| entry.satisfied)
                    && !observed.entries.is_empty(),
                "REP-FILE-005",
            )
        }
        RepositoryFilePredicate::Document(policy) => {
            for entry in &mut observed.entries {
                let bytes = inputs.read(root, entry)?;
                documents::evaluate(policy, &bytes, entry)?;
            }
            (
                policy.assertion == zrail_core::RepositoryDocumentAssertion::Absent
                    || !observed.entries.is_empty(),
                "REP-FILE-007",
            )
        }
    };
    observed.satisfied = satisfied && observed.entries.iter().all(|entry| entry.satisfied);
    Ok(diagnostic)
}

fn forbidden_names(
    path: &str,
    names: &[String],
    part: RepositoryNamePart,
    case: RepositoryCaseMode,
) -> Vec<String> {
    let path = Path::new(path);
    let candidates = match part {
        RepositoryNamePart::Component => path.iter().collect::<Vec<_>>(),
        RepositoryNamePart::ComponentStem => path
            .iter()
            .filter_map(|component| Path::new(component).file_stem())
            .collect(),
        RepositoryNamePart::FileName => path.file_name().into_iter().collect(),
        RepositoryNamePart::FileStem => path.file_stem().into_iter().collect(),
    };
    candidates
        .into_iter()
        .filter_map(|name| name.to_str())
        .filter(|name| {
            names
                .iter()
                .any(|forbidden| literal::fold(name, case) == literal::fold(forbidden, case))
        })
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
