use crate::Number;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

/// An owned JSON object: a string-keyed map of [`Value`]s.
///
/// Backed by a `BTreeMap`, so iteration order is key-sorted rather than
/// insertion order (this matches `serde_json::Value`'s default behavior
/// without its `preserve_order` feature).
pub type Map = BTreeMap<String, Value>;

/// A JSON value: one of the six kinds defined by
/// [RFC 8259 §3](https://www.rfc-editor.org/rfc/rfc8259#section-3).
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// The JSON `null` literal.
    Null,
    /// A JSON boolean, `true` or `false`.
    Bool(bool),
    /// A JSON number.
    Number(Number),
    /// A JSON string.
    String(String),
    /// A JSON array.
    Array(Vec<Value>),
    /// A JSON object.
    Object(Map),
}

impl Default for Value {
    /// Returns `Value::Null`.
    fn default() -> Self {
        Value::Null
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_null() {
        assert_eq!(Value::default(), Value::Null);
    }

    #[test]
    fn variants_are_distinguishable() {
        assert_ne!(Value::Null, Value::Bool(false));
        assert_ne!(Value::Bool(true), Value::Bool(false));
    }
}
