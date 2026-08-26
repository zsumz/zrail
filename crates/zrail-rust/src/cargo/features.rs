//! Cargo feature declarations and local closure are modeled without invoking Cargo.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use toml::Value;

use super::Dependency;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PackageFeatureSet {
    declared: BTreeSet<String>,
    activations: BTreeMap<String, Vec<String>>,
    implicit_dependencies: BTreeSet<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PackageFeatureResolution {
    pub(crate) active: BTreeSet<String>,
    pub(crate) enabled_dependencies: BTreeSet<String>,
    pub(crate) dependency_features: BTreeMap<String, BTreeSet<String>>,
}

impl PackageFeatureSet {
    pub(crate) fn parse(value: &Value, dependencies: &[Dependency]) -> Result<Self, String> {
        let table = value
            .get("features")
            .map_or(Ok(None), |features| {
                features
                    .as_table()
                    .map(Some)
                    .ok_or_else(|| "Cargo [features] must be a table".to_owned())
            })?
            .cloned()
            .unwrap_or_default();
        let mut activations = BTreeMap::new();
        for (name, value) in table {
            validate_name(&name, "feature")?;
            let values = value
                .as_array()
                .ok_or_else(|| format!("Cargo feature {name:?} must be an array"))?;
            let mut entries = values
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .map(str::to_owned)
                        .ok_or_else(|| format!("Cargo feature {name:?} must contain strings"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            entries.sort();
            entries.dedup();
            activations.insert(name, entries);
        }
        let suppressed = activations
            .values()
            .flatten()
            .filter_map(|entry| entry.strip_prefix("dep:"))
            .collect::<BTreeSet<_>>();
        let mut declared = activations.keys().cloned().collect::<BTreeSet<_>>();
        let mut implicit_dependencies = BTreeSet::new();
        for dependency in dependencies.iter().filter(|dependency| dependency.optional) {
            if !suppressed.contains(dependency.alias.as_str()) {
                declared.insert(dependency.alias.clone());
                implicit_dependencies.insert(dependency.alias.clone());
            }
        }
        let result = Self {
            declared,
            activations,
            implicit_dependencies,
        };
        result.validate_activations(dependencies)?;
        Ok(result)
    }

    pub(crate) fn resolve(
        &self,
        default_features: bool,
        selected: &[String],
    ) -> Result<BTreeSet<String>, String> {
        self.resolve_details(default_features, selected)
            .map(|resolved| resolved.active)
    }

    pub(crate) fn resolve_details(
        &self,
        default_features: bool,
        selected: &[String],
    ) -> Result<PackageFeatureResolution, String> {
        let mut result = PackageFeatureResolution::default();
        let mut queue = VecDeque::new();
        if default_features && self.declared.contains("default") {
            queue.push_back("default".to_owned());
        }
        for feature in selected {
            if !self.declared.contains(feature) {
                return Err(format!(
                    "selected Cargo feature {feature:?} is not declared"
                ));
            }
            queue.push_back(feature.clone());
        }
        while let Some(feature) = queue.pop_front() {
            if !result.active.insert(feature.clone()) {
                continue;
            }
            if self.implicit_dependencies.contains(&feature) {
                result.enabled_dependencies.insert(feature.clone());
            }
            for activation in self.activations.get(&feature).into_iter().flatten() {
                if self.declared.contains(activation) {
                    queue.push_back(activation.clone());
                } else if let Some(alias) = activation.strip_prefix("dep:") {
                    result.enabled_dependencies.insert(alias.to_owned());
                } else if let Some((alias, dependency_feature)) = activation.split_once('/') {
                    let weak = alias.ends_with('?');
                    let alias = alias.strip_suffix('?').unwrap_or(alias);
                    if !weak {
                        result.enabled_dependencies.insert(alias.to_owned());
                    }
                    result
                        .dependency_features
                        .entry(alias.to_owned())
                        .or_default()
                        .insert(dependency_feature.to_owned());
                }
            }
        }
        Ok(result)
    }

    pub(crate) fn canonical_definition(&self) -> Vec<String> {
        let mut definition = self
            .declared
            .iter()
            .map(|feature| format!("declare:{feature}"))
            .collect::<Vec<_>>();
        for (feature, activations) in &self.activations {
            definition.extend(
                activations
                    .iter()
                    .map(|activation| format!("activate:{feature}:{activation}")),
            );
        }
        definition
    }

    pub(crate) fn declared(&self) -> &BTreeSet<String> {
        &self.declared
    }

    fn validate_activations(&self, dependencies: &[Dependency]) -> Result<(), String> {
        for (feature, entries) in &self.activations {
            for entry in entries {
                validate_activation(feature, entry, &self.declared, dependencies)?;
            }
        }
        Ok(())
    }
}

fn validate_activation(
    feature: &str,
    activation: &str,
    declared: &BTreeSet<String>,
    dependencies: &[Dependency],
) -> Result<(), String> {
    if let Some(alias) = activation.strip_prefix("dep:") {
        validate_name(alias, "dependency feature")?;
        if dependencies
            .iter()
            .any(|dependency| dependency.alias == alias && dependency.optional)
        {
            return Ok(());
        }
        return Err(format!(
            "Cargo feature {feature:?} activates unknown optional dependency {alias:?}"
        ));
    }
    if let Some((alias, dependency_feature)) = activation.split_once('/') {
        let weak = alias.ends_with('?');
        let alias = alias.strip_suffix('?').unwrap_or(alias);
        validate_name(alias, "dependency feature package")?;
        validate_name(dependency_feature, "dependency feature")?;
        if dependencies
            .iter()
            .any(|dependency| dependency.alias == alias && (!weak || dependency.optional))
        {
            return Ok(());
        }
        return Err(format!(
            "Cargo feature {feature:?} activates unknown dependency {alias:?}"
        ));
    }
    validate_name(activation, "feature activation")?;
    if declared.contains(activation) {
        Ok(())
    } else {
        Err(format!(
            "Cargo feature {feature:?} activates unknown feature {activation:?}"
        ))
    }
}

fn validate_name(value: &str, label: &str) -> Result<(), String> {
    let mut characters = value.chars();
    let first_valid = characters.next().is_some_and(|character| {
        character == '_' || character.is_ascii_digit() || unicode_ident::is_xid_start(character)
    });
    let rest_valid = characters.all(|character| {
        unicode_ident::is_xid_continue(character) || matches!(character, '-' | '+' | '.')
    });
    if first_valid && rest_valid {
        Ok(())
    } else {
        Err(format!("Cargo {label} name {value:?} is invalid"))
    }
}

#[cfg(test)]
#[path = "features_test.rs"]
mod features_test;
