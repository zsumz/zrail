//! Lock mutations retain exact resolvable incoming edges so inventory failures stay attributable.

use toml::Value;

pub(super) fn cases(original: &[u8], name: &str) -> Vec<(String, Vec<u8>, bool)> {
    let original: Value =
        toml::from_str(std::str::from_utf8(original).expect("UTF-8 lock")).expect("original lock");
    let selected = packages(&original)
        .iter()
        .find(|node| node["name"].as_str() == Some(name))
        .expect("frozen selected node");
    let version = selected["version"].as_str().expect("locked version");
    let source = selected["source"].as_str().expect("registry source");
    let mut cases = Vec::new();
    let mut removed = original.clone();
    packages_mut(&mut removed).retain(|node| node["name"].as_str() != Some(name));
    edges(&mut removed, name, version, source, None);
    cases.push(encoded("required-node-removed-with-edges", &removed, false));
    for (case, field, value) in [
        ("same-count-version", "version", "99.0.0".to_owned()),
        (
            "same-count-source",
            "source",
            "registry+https://unreviewed.invalid/index".to_owned(),
        ),
        ("same-count-checksum", "checksum", "f".repeat(64)),
    ] {
        let mut changed = original.clone();
        let target = packages_mut(&mut changed)
            .iter_mut()
            .find(|node| node["name"].as_str() == Some(name))
            .expect("selected node");
        target
            .as_table_mut()
            .expect("node table")
            .insert(field.into(), Value::String(value.clone()));
        let new_version = if field == "version" {
            value.as_str()
        } else {
            version
        };
        let new_source = if field == "source" {
            value.as_str()
        } else {
            source
        };
        edges(
            &mut changed,
            name,
            version,
            source,
            Some(&format!("{name} {new_version} ({new_source})")),
        );
        cases.push(encoded(case, &changed, false));
    }
    for (case, field, value) in [
        ("duplicate-name-new-version", "version", "99.0.0"),
        (
            "duplicate-name-new-source",
            "source",
            "registry+https://unreviewed.invalid/index",
        ),
    ] {
        let mut changed = original.clone();
        edges(
            &mut changed,
            name,
            version,
            source,
            Some(&format!("{name} {version} ({source})")),
        );
        let mut duplicate = selected.clone();
        duplicate
            .as_table_mut()
            .expect("node")
            .insert(field.into(), Value::String(value.into()));
        packages_mut(&mut changed).push(duplicate);
        cases.push(encoded(case, &changed, false));
    }
    let mut reordered = original.clone();
    packages_mut(&mut reordered).reverse();
    cases.push(encoded("reordered-nodes", &reordered, true));
    let mut extra = original.clone();
    let mut unrelated = selected.clone();
    let table = unrelated.as_table_mut().expect("unrelated table");
    table.insert(
        "name".into(),
        Value::String("unrelated-parity-package".into()),
    );
    table.remove("dependencies");
    packages_mut(&mut extra).push(unrelated);
    cases.push(encoded("unselected-valid-node", &extra, true));
    let mut qualified = original.clone();
    edges(
        &mut qualified,
        name,
        version,
        source,
        Some(&format!("{name} {version} ({source})")),
    );
    cases.push(encoded("qualified-incoming-edges", &qualified, true));
    cases
}

fn edges(document: &mut Value, name: &str, version: &str, source: &str, replacement: Option<&str>) {
    for node in packages_mut(document) {
        let Some(dependencies) = node.get_mut("dependencies").and_then(Value::as_array_mut) else {
            continue;
        };
        dependencies.retain_mut(|entry| {
            let value = entry.as_str().expect("locked dependency reference");
            if value == name
                || value == format!("{name} {version}")
                || value == format!("{name} {version} ({source})")
            {
                if let Some(replacement) = replacement {
                    *entry = Value::String(replacement.into());
                } else {
                    return false;
                }
            }
            true
        });
    }
}

fn packages(document: &Value) -> &Vec<Value> {
    document["package"].as_array().expect("package array")
}

fn packages_mut(document: &mut Value) -> &mut Vec<Value> {
    document
        .get_mut("package")
        .and_then(Value::as_array_mut)
        .expect("package array")
}

fn encoded(name: &str, document: &Value, accepted: bool) -> (String, Vec<u8>, bool) {
    (
        name.into(),
        toml::to_string(document)
            .expect("mutated Cargo.lock")
            .into_bytes(),
        accepted,
    )
}
