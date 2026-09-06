//! A borrowed typed view preserves TOML/JSON distinctions without inventing resolved values.

use zrail_core::RepositoryDocumentValue;

#[derive(Clone, Copy)]
pub(super) enum Node<'a> {
    Toml(&'a toml::Value),
    Json(&'a serde_json::Value),
}

impl<'a> Node<'a> {
    pub(super) fn keys(self) -> Option<Vec<&'a str>> {
        match self {
            Self::Toml(toml::Value::Table(table)) => {
                Some(table.keys().map(String::as_str).collect())
            }
            Self::Json(serde_json::Value::Object(object)) => {
                Some(object.keys().map(String::as_str).collect())
            }
            _ => None,
        }
    }
    pub(super) fn select(mut self, keys: &[String]) -> Result<Option<Self>, String> {
        for (index, key) in keys.iter().enumerate() {
            let child = match self {
                Self::Toml(toml::Value::Table(table)) => table.get(key).map(Self::Toml),
                Self::Json(serde_json::Value::Object(object)) => object.get(key).map(Self::Json),
                _ => {
                    return Err(format!(
                        "key {index} {key:?} requires an object/table, found {}",
                        self.kind()
                    ));
                }
            };
            let Some(child) = child else { return Ok(None) };
            self = child;
        }
        Ok(Some(self))
    }

    pub(super) fn kind(self) -> &'static str {
        match self {
            Self::Toml(toml::Value::String(_)) | Self::Json(serde_json::Value::String(_)) => {
                "string"
            }
            Self::Toml(toml::Value::Integer(_)) => "integer",
            Self::Json(serde_json::Value::Number(value)) if value.is_i64() || value.is_u64() => {
                "integer"
            }
            Self::Toml(toml::Value::Float(_)) | Self::Json(serde_json::Value::Number(_)) => "float",
            Self::Toml(toml::Value::Boolean(_)) | Self::Json(serde_json::Value::Bool(_)) => {
                "boolean"
            }
            Self::Toml(toml::Value::Datetime(_)) => "datetime",
            Self::Toml(toml::Value::Array(_)) | Self::Json(serde_json::Value::Array(_)) => "array",
            Self::Toml(toml::Value::Table(_)) | Self::Json(serde_json::Value::Object(_)) => {
                "object"
            }
            Self::Json(serde_json::Value::Null) => "null",
        }
    }

    pub(super) fn string(self) -> Option<&'a str> {
        match self {
            Self::Toml(value) => value.as_str(),
            Self::Json(value) => value.as_str(),
        }
    }

    pub(super) fn value(self) -> Option<RepositoryDocumentValue> {
        match self {
            Self::Toml(toml::Value::String(value))
            | Self::Json(serde_json::Value::String(value)) => {
                Some(RepositoryDocumentValue::String(value.clone()))
            }
            Self::Toml(toml::Value::Boolean(value))
            | Self::Json(serde_json::Value::Bool(value)) => {
                Some(RepositoryDocumentValue::Boolean(*value))
            }
            Self::Toml(toml::Value::Integer(value)) => {
                Some(RepositoryDocumentValue::Integer(*value))
            }
            Self::Json(serde_json::Value::Number(value)) => {
                value.as_i64().map(RepositoryDocumentValue::Integer)
            }
            Self::Toml(toml::Value::Array(values)) => values
                .iter()
                .map(|value| value.as_str().map(str::to_owned))
                .collect::<Option<Vec<_>>>()
                .map(RepositoryDocumentValue::Strings),
            Self::Json(serde_json::Value::Array(values)) => values
                .iter()
                .map(|value| value.as_str().map(str::to_owned))
                .collect::<Option<Vec<_>>>()
                .map(RepositoryDocumentValue::Strings),
            _ => None,
        }
    }

    pub(super) fn check_bounds(self) -> Result<(), String> {
        let mut pending = vec![(self, 0)];
        let mut count = 0;
        while let Some((node, depth)) = pending.pop() {
            count += 1;
            if count > 100_000 || depth > 64 {
                return Err("document exceeds the 100000-node or 64-level safety limit".into());
            }
            match node {
                Self::Toml(toml::Value::Array(values)) => {
                    pending.extend(values.iter().map(|value| (Self::Toml(value), depth + 1)));
                }
                Self::Toml(toml::Value::Table(values)) => {
                    pending.extend(values.values().map(|value| (Self::Toml(value), depth + 1)));
                }
                Self::Json(serde_json::Value::Array(values)) => {
                    pending.extend(values.iter().map(|value| (Self::Json(value), depth + 1)));
                }
                Self::Json(serde_json::Value::Object(values)) => {
                    pending.extend(values.values().map(|value| (Self::Json(value), depth + 1)));
                }
                _ => {}
            }
        }
        Ok(())
    }
}
