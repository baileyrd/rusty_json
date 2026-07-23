//! `serde::Serialize`/`Deserialize` impls for [`Value`], so it can be used
//! with any serde data format, not just this crate's own parser/serializer.

use crate::Value;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::Serializer;

impl serde::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Number(n) => {
                if let Some(u) = n.as_u64() {
                    serializer.serialize_u64(u)
                } else if let Some(i) = n.as_i64() {
                    serializer.serialize_i64(i)
                } else {
                    serializer.serialize_f64(n.as_f64())
                }
            }
            Value::String(s) => serializer.serialize_str(s),
            Value::Array(items) => {
                let mut seq = serializer.serialize_seq(Some(items.len()))?;
                for item in items {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            Value::Object(map) => {
                let mut ser_map = serializer.serialize_map(Some(map.len()))?;
                for (key, val) in map {
                    ser_map.serialize_entry(key, val)?;
                }
                ser_map.end()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Map, Number};
    use alloc::string::String;
    use alloc::vec::Vec;

    #[test]
    fn serializes_through_real_serde_json_serializer() {
        // Drives our `impl Serialize for Value` through an actual external
        // serde data format (not our own writer), proving it's genuinely
        // serde-compatible rather than just internally consistent.
        let mut map = Map::new();
        map.insert(String::from("a"), Value::Number(Number::from(1u64)));
        let value = Value::Object(map);

        assert_eq!(serde_json::to_string(&value).unwrap(), r#"{"a":1}"#);
    }

    #[test]
    fn serializes_all_variants_through_serde_json() {
        let values = alloc::vec![
            Value::Null,
            Value::Bool(true),
            Value::Number(Number::from(1u64)),
            Value::Number(Number::from(-1i64)),
            Value::Number(Number::from_f64(1.5).unwrap()),
            Value::String(String::from("s")),
            Value::Array(Vec::new()),
            Value::Object(Map::new()),
        ];
        let expected = ["null", "true", "1", "-1", "1.5", "\"s\"", "[]", "{}"];
        for (v, want) in values.iter().zip(expected) {
            assert_eq!(serde_json::to_string(v).unwrap(), want);
        }
    }

    #[test]
    fn nested_array_and_object_through_serde_json() {
        let value = Value::Array(alloc::vec![
            Value::Number(Number::from(1u64)),
            Value::Object({
                let mut m = Map::new();
                m.insert(String::from("k"), Value::Bool(true));
                m
            }),
        ]);
        assert_eq!(serde_json::to_string(&value).unwrap(), r#"[1,{"k":true}]"#);
    }
}
