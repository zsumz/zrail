//! Complete canonical lock surfaces used by epoch migration review.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::LockFile;

pub(super) fn surfaces(
    lock: &LockFile,
) -> Result<BTreeMap<(String, String), String>, serde_json::Error> {
    let mut values = BTreeMap::new();
    insert(
        &mut values,
        "lock.contract",
        "zrail.toml",
        &lock.contract_sha256,
    );
    analysis(lock, &mut values);
    packages(lock, &mut values)?;
    for generated in &lock.generated {
        insert(
            &mut values,
            "rust.generated-provenance",
            &generated.root,
            &generated.manifest_sha256,
        );
    }
    for gate in &lock.gates {
        insert_serialized(&mut values, "gate.lock", &gate.name, gate)?;
    }
    for receipt in &lock.execution_receipts {
        insert_serialized(
            &mut values,
            "rust.test-mirror-receipt-lock",
            &receipt.production,
            receipt,
        )?;
    }
    macros(lock, &mut values)?;
    for ratchet in &lock.ratchets {
        let subject = format!(
            "{}[{}]:{}",
            ratchet.rule,
            ratchet.selector.as_deref().unwrap_or(""),
            ratchet.target
        );
        insert_serialized(&mut values, "ratchet", &subject, ratchet)?;
    }
    Ok(values)
}

fn analysis(lock: &LockFile, values: &mut BTreeMap<(String, String), String>) {
    let Some(analysis) = &lock.analysis else {
        return;
    };
    insert(
        values,
        "analysis.inventory",
        "repository",
        &analysis.inventory_sha256,
    );
    insert(
        values,
        "analysis.exclusions",
        "repository",
        &analysis.exclusions_sha256,
    );
    if let Some(digest) = &analysis.cargo_lock_sha256 {
        insert(values, "analysis.cargo-lock", "Cargo.lock", digest);
    }
    if !analysis.cargo_features_sha256.is_empty() {
        insert(
            values,
            "analysis.cargo-features",
            "workspace",
            &analysis.cargo_features_sha256,
        );
    }
    if !analysis.feature_worlds_sha256.is_empty() {
        insert(
            values,
            "analysis.feature-worlds",
            "workspace",
            &analysis.feature_worlds_sha256,
        );
        if let Some(count) = analysis.feature_worlds {
            insert(
                values,
                "analysis.feature-world-count",
                "workspace",
                &count.to_string(),
            );
        }
    }
    for source in &analysis.contract_sources {
        insert(
            values,
            "analysis.contract-source",
            &source.path,
            &source.sha256,
        );
    }
}

fn packages(
    lock: &LockFile,
    values: &mut BTreeMap<(String, String), String>,
) -> Result<(), serde_json::Error> {
    for package in &lock.packages {
        insert(values, "repository.package", &package.name, &package.name);
        for dependency in &package.dependencies {
            let subject = format!(
                "{}:{}:{}:{:?}:{}",
                package.name,
                dependency.alias.as_deref().unwrap_or(""),
                dependency.name,
                dependency.kind,
                dependency.target.as_deref().unwrap_or("")
            );
            insert_serialized(values, "dependency.resolved-edge", &subject, dependency)?;
        }
    }
    Ok(())
}

fn macros(
    lock: &LockFile,
    values: &mut BTreeMap<(String, String), String>,
) -> Result<(), serde_json::Error> {
    for implementation in &lock.macro_implementations {
        let subject = format!("{}:{}", implementation.directory, implementation.package);
        insert_serialized(
            values,
            "rust.macro-implementation",
            &subject,
            implementation,
        )?;
    }
    for source in &lock.macro_sources {
        insert_serialized(values, "rust.macro-source", &source.allowance, source)?;
    }
    for manifest in &lock.item_macro_manifests {
        let subject = format!("{}:{}", manifest.name, manifest.invocation_path);
        insert_serialized(values, "rust.item-macro-manifest", &subject, manifest)?;
    }
    Ok(())
}

fn insert(values: &mut BTreeMap<(String, String), String>, rail: &str, subject: &str, value: &str) {
    values.insert((rail.into(), subject.into()), value.into());
}

fn insert_serialized<T: Serialize>(
    values: &mut BTreeMap<(String, String), String>,
    rail: &str,
    subject: &str,
    value: &T,
) -> Result<(), serde_json::Error> {
    let value = serde_json::to_string(value)?;
    insert(values, rail, subject, &value);
    Ok(())
}
