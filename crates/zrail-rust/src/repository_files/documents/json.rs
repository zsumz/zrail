//! Strict bounded JSON preserves all keys and rejects duplicates before they can be overwritten.

use std::{cell::Cell, fmt};

use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};

pub(super) fn parse(text: &str) -> Result<Value, String> {
    let count = Cell::new(0);
    let mut parser = serde_json::Deserializer::from_str(text);
    let value = Seed {
        depth: 0,
        count: &count,
    }
    .deserialize(&mut parser)
    .map_err(|error| format!("invalid JSON: {error}"))?;
    parser
        .end()
        .map_err(|error| format!("invalid JSON: {error}"))?;
    Ok(value)
}

struct Seed<'a> {
    depth: usize,
    count: &'a Cell<usize>,
}

impl<'de> DeserializeSeed<'de> for Seed<'_> {
    type Value = Value;

    fn deserialize<D: serde::Deserializer<'de>>(self, parser: D) -> Result<Value, D::Error> {
        self.count.set(self.count.get() + 1);
        if self.depth > 64 || self.count.get() > 100_000 {
            return Err(D::Error::custom(
                "document exceeds the 100000-node or 64-level safety limit",
            ));
        }
        parser.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Seed<'_> {
    type Value = Value;

    fn expecting(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output.write_str("bounded JSON with unique object keys")
    }

    fn visit_bool<E: Error>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E: Error>(self, value: i64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_u64<E: Error>(self, value: u64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_f64<E: Error>(self, value: f64) -> Result<Value, E> {
        Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E: Error>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.into()))
    }

    fn visit_unit<E: Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(Seed {
            depth: self.depth + 1,
            count: self.count,
        })? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut mapping: A) -> Result<Value, A::Error> {
        let mut values = Map::new();
        while let Some(key) = mapping.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(A::Error::custom(format!(
                    "duplicate JSON object key {key:?}"
                )));
            }
            let value = mapping.next_value_seed(Seed {
                depth: self.depth + 1,
                count: self.count,
            })?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}
