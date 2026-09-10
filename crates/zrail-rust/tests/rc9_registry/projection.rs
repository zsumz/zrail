//! Observe every deserialized field without modifying the extracted original types.

use super::original::Guardrails;
use serde::Serialize;
use serde_json::Value;

fn scalar<T: Serialize>(value: T) -> Value {
    serde_json::to_value(value).expect("typed registry field")
}

fn object<const N: usize>(fields: [(&str, Value); N]) -> Value {
    Value::Object(
        fields
            .into_iter()
            .map(|(key, value)| (key.into(), value))
            .collect(),
    )
}

pub(super) fn value(config: Guardrails) -> Value {
    let dependencies = config.dependencies;
    object([
        ("schema", scalar(config.schema)),
        (
            "paths",
            object([("rust_roots", scalar(config.paths.rust_roots))]),
        ),
        (
            "budgets",
            object([
                ("facade", scalar(config.budgets.facade)),
                ("production", scalar(config.budgets.production)),
                ("test", scalar(config.budgets.test)),
            ]),
        ),
        (
            "capabilities",
            Value::Array(
                config
                    .capabilities
                    .into_iter()
                    .map(|entry| {
                        object([
                            ("root", scalar(entry.root)),
                            ("forbidden", scalar(entry.forbidden)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "dependencies",
            object([
                ("banned", scalar(dependencies.banned)),
                ("core_allowed", scalar(dependencies.core_allowed)),
                ("driver_allowed", scalar(dependencies.driver_allowed)),
                ("probe_allowed", scalar(dependencies.probe_allowed)),
                ("transport_allowed", scalar(dependencies.transport_allowed)),
                (
                    "kafka_wire_version",
                    scalar(dependencies.kafka_wire_version),
                ),
                (
                    "kafka_wire_checksum",
                    scalar(dependencies.kafka_wire_checksum),
                ),
                (
                    "kafka_wire_core_version",
                    scalar(dependencies.kafka_wire_core_version),
                ),
                (
                    "kafka_wire_core_checksum",
                    scalar(dependencies.kafka_wire_core_checksum),
                ),
                ("bornera_version", scalar(dependencies.bornera_version)),
                ("bornera_checksum", scalar(dependencies.bornera_checksum)),
                (
                    "bornera_core_version",
                    scalar(dependencies.bornera_core_version),
                ),
                (
                    "bornera_core_checksum",
                    scalar(dependencies.bornera_core_checksum),
                ),
                (
                    "bornera_rustls_version",
                    scalar(dependencies.bornera_rustls_version),
                ),
                (
                    "bornera_rustls_checksum",
                    scalar(dependencies.bornera_rustls_checksum),
                ),
            ]),
        ),
    ])
}
