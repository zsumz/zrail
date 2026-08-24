//! Manifest dependency declarations map to one exact outgoing Cargo.lock edge.

use semver::{Version, VersionReq};

use crate::cargo::{Dependency, DependencySource, Package};

use super::{ResolvedCargoGraph, ResolvedPackageIdentity};

impl ResolvedCargoGraph {
    pub(crate) fn source_matches(
        &self,
        identity: &ResolvedPackageIdentity,
        source: &DependencySource,
    ) -> Result<bool, String> {
        if !self.packages.contains_key(identity) {
            return Err(format!(
                "resolved identity {} does not belong to this Cargo.lock graph",
                identity.label()
            ));
        }
        matches_source(identity, source)
    }

    pub(crate) fn manifest_dependency(
        &self,
        package: &Package,
        dependency: &Dependency,
    ) -> Result<ResolvedPackageIdentity, String> {
        let source = self.workspace_package(&package.name).ok_or_else(|| {
            format!(
                "Cargo.lock contains no local node for workspace package {:?}",
                package.name
            )
        })?;
        let candidates = self
            .dependencies(source)
            .iter()
            .filter(|candidate| candidate.name == dependency.name)
            .filter_map(|candidate| {
                matches_source(candidate, &dependency.source)
                    .map(|matches| matches.then_some(candidate))
                    .transpose()
            })
            .collect::<Result<Vec<_>, _>>()?;
        match candidates.as_slice() {
            [candidate] => Ok((*candidate).clone()),
            [] => Err(format!(
                "manifest dependency {:?} from package {:?} maps to no outgoing Cargo.lock node",
                dependency.alias, package.name
            )),
            _ => Err(format!(
                "manifest dependency {:?} from package {:?} maps ambiguously to {} Cargo.lock nodes: {}",
                dependency.alias,
                package.name,
                candidates.len(),
                candidates
                    .iter()
                    .map(|candidate| candidate.label())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }
}

fn matches_source(
    candidate: &ResolvedPackageIdentity,
    source: &DependencySource,
) -> Result<bool, String> {
    match source {
        DependencySource::WorkspaceMember {
            directory,
            requirement,
        }
        | DependencySource::RepositoryPath {
            path: directory,
            requirement,
        } => Ok(candidate.source == format!("path+{directory}")
            && matches_requirement(&candidate.version, requirement.as_deref())?),
        DependencySource::Registry {
            registry,
            index,
            requirement,
        } => {
            let expected = match (registry.as_deref(), index.as_deref()) {
                (None, None) => "registry+https://github.com/rust-lang/crates.io-index",
                (None, Some(index)) => {
                    return Ok(candidate.source == format!("registry+{index}")
                        && matches_requirement(&candidate.version, Some(requirement))?);
                }
                (Some(name), None) => {
                    return Err(format!(
                        "named Cargo registry {name:?} has no exact registry-index for locked resolution"
                    ));
                }
                (Some(_), Some(_)) => {
                    return Err("Cargo registry and registry-index are mutually exclusive".into());
                }
            };
            Ok(candidate.source == expected
                && matches_requirement(&candidate.version, Some(requirement))?)
        }
        DependencySource::Git {
            repository,
            requirement,
            ..
        } => {
            let source = candidate.source.strip_prefix("git+");
            let same_repository = source.is_some_and(|source| {
                source == repository
                    || source
                        .strip_prefix(repository)
                        .is_some_and(|suffix| suffix.starts_with('?') || suffix.starts_with('#'))
            });
            Ok(same_repository && matches_requirement(&candidate.version, requirement.as_deref())?)
        }
    }
}

fn matches_requirement(version: &str, requirement: Option<&str>) -> Result<bool, String> {
    let Some(requirement) = requirement else {
        return Ok(true);
    };
    let requirement = VersionReq::parse(requirement).map_err(|error| {
        format!("Cargo version requirement {requirement:?} cannot be resolved: {error}")
    })?;
    let version = Version::parse(version)
        .map_err(|error| format!("Cargo.lock version {version:?} is invalid: {error}"))?;
    Ok(requirement.matches(&version))
}
