use crate::Value;
use alloc::string::{String, ToString};

/// Serializes a [`Value`] to a compact JSON string.
pub fn to_string(value: &Value) -> String {
    let mut out = String::new();
    write_value(value, &mut out);
    out
}

fn write_value(value: &Value, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(n) => out.push_str(&n.to_string()),
        Value::String(s) => write_escaped_string(s, out),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_value(item, out);
            }
            out.push(']');
        }
        Value::Object(map) => {
            out.push('{');
            for (i, (key, val)) in map.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_escaped_string(key, out);
                out.push(':');
                write_value(val, out);
            }
            out.push('}');
        }
    }
}

fn write_escaped_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000C}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&alloc::format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_str, Map, Number};
    use alloc::string::String as AString;
    use alloc::vec::Vec;

    #[test]
    fn serializes_scalars() {
        assert_eq!(to_string(&Value::Null), "null");
        assert_eq!(to_string(&Value::Bool(true)), "true");
        assert_eq!(to_string(&Value::Bool(false)), "false");
        assert_eq!(to_string(&Value::Number(Number::from(42u64))), "42");
        assert_eq!(to_string(&Value::String(AString::from("hi"))), "\"hi\"");
    }

    #[test]
    fn escapes_special_characters() {
        assert_eq!(
            to_string(&Value::String(AString::from("a\"b\\c\nd\te"))),
            r#""a\"b\\c\nd\te""#
        );
        assert_eq!(
            to_string(&Value::String(AString::from("\u{0001}"))),
            "\"\\u0001\""
        );
    }

    #[test]
    fn serializes_array_and_object() {
        assert_eq!(to_string(&Value::Array(Vec::new())), "[]");
        assert_eq!(to_string(&Value::Object(Map::new())), "{}");

        let arr = Value::Array(alloc::vec![
            Value::Number(Number::from(1u64)),
            Value::Bool(true)
        ]);
        assert_eq!(to_string(&arr), "[1,true]");

        let mut map = Map::new();
        map.insert(AString::from("a"), Value::Number(Number::from(1u64)));
        assert_eq!(to_string(&Value::Object(map)), r#"{"a":1}"#);
    }

    #[test]
    fn round_trips_through_parser() {
        let inputs = [
            r#"null"#,
            r#"true"#,
            r#"42"#,
            r#"-1.5"#,
            r#""hello\nworld""#,
            r#"[1,2,3]"#,
            r#"{"a":1,"b":[true,null]}"#,
        ];
        for input in inputs {
            let value = from_str(input).unwrap();
            let serialized = to_string(&value);
            let reparsed = from_str(&serialized).unwrap();
            assert_eq!(value, reparsed, "round-trip failed for {input}");
        }
    }
}
