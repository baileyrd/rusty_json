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

impl Value {
    /// Looks up a key if this is an object, returning `None` otherwise
    /// (including when the key is absent).
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(map) => map.get(key),
            _ => None,
        }
    }

    /// Looks up an index if this is an array, returning `None` otherwise
    /// (including when the index is out of bounds).
    pub fn get_index(&self, index: usize) -> Option<&Value> {
        match self {
            Value::Array(arr) => arr.get(index),
            _ => None,
        }
    }

    /// True if this is `Value::Null`.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// True if this is `Value::Bool`.
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    /// True if this is `Value::Number`.
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    /// True if this is `Value::String`.
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// True if this is `Value::Array`.
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    /// True if this is `Value::Object`.
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    /// Returns the inner `bool`, if this is `Value::Bool`.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns the inner string slice, if this is `Value::String`.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the inner array, if this is `Value::Array`.
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Returns the inner object, if this is `Value::Object`.
    pub fn as_object(&self) -> Option<&Map> {
        match self {
            Value::Object(m) => Some(m),
            _ => None,
        }
    }

    /// Returns the number as an `i64`, if this is `Value::Number` and it
    /// fits without loss. See [`Number::as_i64`].
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Number(n) => n.as_i64(),
            _ => None,
        }
    }

    /// Returns the number as a `u64`, if this is `Value::Number` and it
    /// fits without loss. See [`Number::as_u64`].
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Number(n) => n.as_u64(),
            _ => None,
        }
    }

    /// Returns the number as an `f64`, if this is `Value::Number`. See
    /// [`Number::as_f64`].
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(n.as_f64()),
            _ => None,
        }
    }
}

impl core::ops::Index<&str> for Value {
    type Output = Value;

    /// Indexes into an object by key. Returns `&Value::Null` if this isn't
    /// an object or the key is absent, rather than panicking.
    fn index(&self, key: &str) -> &Value {
        static NULL: Value = Value::Null;
        self.get(key).unwrap_or(&NULL)
    }
}

impl core::ops::Index<usize> for Value {
    type Output = Value;

    /// Indexes into an array by position. Returns `&Value::Null` if this
    /// isn't an array or the index is out of bounds, rather than panicking.
    fn index(&self, index: usize) -> &Value {
        static NULL: Value = Value::Null;
        self.get_index(index).unwrap_or(&NULL)
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

    #[test]
    fn is_predicates() {
        assert!(Value::Null.is_null());
        assert!(Value::Bool(true).is_bool());
        assert!(Value::Number(Number::from(1u64)).is_number());
        assert!(Value::String(String::from("x")).is_string());
        assert!(Value::Array(Vec::new()).is_array());
        assert!(Value::Object(Map::new()).is_object());
        assert!(!Value::Null.is_bool());
    }

    #[test]
    fn as_accessors_match_and_mismatch() {
        assert_eq!(Value::Bool(true).as_bool(), Some(true));
        assert_eq!(Value::Null.as_bool(), None);
        assert_eq!(Value::String(String::from("hi")).as_str(), Some("hi"));
        assert_eq!(Value::Null.as_str(), None);
        assert_eq!(Value::Number(Number::from(7u64)).as_i64(), Some(7));
        assert_eq!(Value::Null.as_i64(), None);
        assert_eq!(
            Value::Number(Number::from_f64(1.5).unwrap()).as_f64(),
            Some(1.5)
        );
    }

    #[test]
    fn get_on_object_and_array() {
        let mut map = Map::new();
        map.insert(String::from("a"), Value::Bool(true));
        let obj = Value::Object(map);
        assert_eq!(obj.get("a"), Some(&Value::Bool(true)));
        assert_eq!(obj.get("missing"), None);
        assert_eq!(obj.get_index(0), None);

        let arr = Value::Array(alloc::vec![Value::Bool(false)]);
        assert_eq!(arr.get_index(0), Some(&Value::Bool(false)));
        assert_eq!(arr.get_index(1), None);
        assert_eq!(arr.get("a"), None);
    }

    #[test]
    fn index_operator_returns_null_instead_of_panicking() {
        let mut map = Map::new();
        map.insert(String::from("a"), Value::Bool(true));
        let obj = Value::Object(map);
        assert_eq!(obj["a"], Value::Bool(true));
        assert_eq!(obj["missing"], Value::Null);
        assert_eq!(obj[0], Value::Null);

        let arr = Value::Array(alloc::vec![Value::Bool(false)]);
        assert_eq!(arr[0], Value::Bool(false));
        assert_eq!(arr[5], Value::Null);
    }
}
