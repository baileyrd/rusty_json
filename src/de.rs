//! A `serde::Deserializer` driving this crate's own hand-rolled parser
//! (`src/parser.rs`), so any `Deserialize` type can be deserialized
//! directly from JSON text, not just [`crate::Value`].

use crate::parser::Parser;
use crate::{Error, Value};
use alloc::string::String;
use serde::de::{
    self, Deserialize, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};

/// Deserializes an instance of `T` from a string slice.
pub fn from_str<'de, T>(s: &'de str) -> Result<T, Error>
where
    T: Deserialize<'de>,
{
    let mut parser = Parser::new(s);
    let value = T::deserialize(&mut parser)?;
    parser.skip_whitespace();
    if !parser.at_end() {
        return Err(parser.error("trailing characters after JSON value"));
    }
    Ok(value)
}

/// Deserializes an instance of `T` from a byte slice.
pub fn from_slice<'de, T>(v: &'de [u8]) -> Result<T, Error>
where
    T: Deserialize<'de>,
{
    let s = core::str::from_utf8(v).map_err(|_| Error::new("input is not valid utf-8", 1, 1))?;
    from_str(s)
}

fn consume_literal(parser: &mut Parser, literal: &str) -> Result<(), Error> {
    // Matches the original hand-rolled parser's classification: running out
    // of input mid-literal is a syntax error, not EOF (an incomplete
    // literal like `nul` is unambiguously wrong, unlike a value that's cut
    // off at a point where more input could still make it valid).
    for expected in literal.bytes() {
        match parser.bump() {
            Some(byte) if byte == expected => {}
            _ => return Err(parser.error(alloc::format!("invalid literal, expected `{literal}`"))),
        }
    }
    Ok(())
}

fn parse_seq<'de, V>(parser: &mut Parser<'de>, visitor: V) -> Result<V::Value, Error>
where
    V: Visitor<'de>,
{
    parser.bump(); // opening `[`
    let value = visitor.visit_seq(CollAccess {
        parser,
        first: true,
    })?;
    parser.skip_whitespace();
    match parser.bump() {
        Some(b']') => Ok(value),
        None => Err(parser.error_eof("unexpected end of input in array")),
        _ => Err(parser.error("expected `,` or `]` in array")),
    }
}

fn parse_map<'de, V>(parser: &mut Parser<'de>, visitor: V) -> Result<V::Value, Error>
where
    V: Visitor<'de>,
{
    parser.bump(); // opening `{`
    let value = visitor.visit_map(CollAccess {
        parser,
        first: true,
    })?;
    parser.skip_whitespace();
    match parser.bump() {
        Some(b'}') => Ok(value),
        None => Err(parser.error_eof("unexpected end of input in object")),
        _ => Err(parser.error("expected `,` or `}` in object")),
    }
}

impl<'de> de::Deserializer<'de> for &mut Parser<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.skip_whitespace();
        match self.peek() {
            Some(b'n') => {
                consume_literal(self, "null")?;
                visitor.visit_unit()
            }
            Some(b't') => {
                consume_literal(self, "true")?;
                visitor.visit_bool(true)
            }
            Some(b'f') => {
                consume_literal(self, "false")?;
                visitor.visit_bool(false)
            }
            Some(b'-' | b'0'..=b'9') => match self.parse_number()? {
                Value::Number(n) => {
                    if let Some(u) = n.as_u64() {
                        visitor.visit_u64(u)
                    } else if let Some(i) = n.as_i64() {
                        visitor.visit_i64(i)
                    } else {
                        visitor.visit_f64(n.as_f64())
                    }
                }
                _ => unreachable!("parse_number always returns Value::Number"),
            },
            Some(b'"') => visitor.visit_string(self.parse_string()?),
            Some(b'[') => parse_seq(self, visitor),
            Some(b'{') => parse_map(self, visitor),
            Some(other) => {
                Err(self.error(alloc::format!("unexpected character `{}`", other as char)))
            }
            None => Err(self.error_eof("unexpected end of input")),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.skip_whitespace();
        if self.peek() == Some(b'n') {
            consume_literal(self, "null")?;
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.skip_whitespace();
        consume_literal(self, "null")?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.skip_whitespace();
        if self.peek() != Some(b'[') {
            return Err(self.error("expected array"));
        }
        parse_seq(self, visitor)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.skip_whitespace();
        if self.peek() != Some(b'{') {
            return Err(self.error("expected object"));
        }
        parse_map(self, visitor)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.skip_whitespace();
        match self.peek() {
            Some(b'"') => {
                let variant = self.parse_string()?;
                visitor.visit_enum(variant.into_deserializer())
            }
            Some(b'{') => {
                self.bump();
                self.skip_whitespace();
                match self.peek() {
                    Some(b'"') => {}
                    None => return Err(self.error_eof("unexpected end of input in enum")),
                    _ => return Err(self.error("expected variant name string")),
                }
                let variant = self.parse_string()?;
                self.skip_whitespace();
                match self.bump() {
                    Some(b':') => {}
                    None => return Err(self.error_eof("unexpected end of input in enum")),
                    _ => return Err(self.error("expected `:` after variant name")),
                }
                let value = visitor.visit_enum(EnumDeserializer {
                    parser: self,
                    variant,
                })?;
                self.skip_whitespace();
                match self.bump() {
                    Some(b'}') => Ok(value),
                    None => Err(self.error_eof("unexpected end of input in enum")),
                    _ => Err(self.error("expected `}` after enum value")),
                }
            }
            Some(_) => Err(self.error("expected string or object for enum")),
            None => Err(self.error_eof("unexpected end of input")),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf identifier
    }
}

/// Drives both `SeqAccess` (arrays) and `MapAccess` (objects): a
/// comma-separated sequence between an already-consumed opening bracket and
/// a closing one the caller consumes after `visit_seq`/`visit_map` returns.
struct CollAccess<'a, 'de> {
    parser: &'a mut Parser<'de>,
    first: bool,
}

impl<'de> SeqAccess<'de> for CollAccess<'_, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        self.parser.skip_whitespace();
        if self.parser.peek() == Some(b']') {
            return Ok(None);
        }
        if !self.first {
            match self.parser.bump() {
                Some(b',') => self.parser.skip_whitespace(),
                None => return Err(self.parser.error_eof("unexpected end of input in array")),
                _ => return Err(self.parser.error("expected `,` or `]` in array")),
            }
        }
        self.first = false;
        seed.deserialize(&mut *self.parser).map(Some)
    }
}

impl<'de> MapAccess<'de> for CollAccess<'_, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: DeserializeSeed<'de>,
    {
        self.parser.skip_whitespace();
        if self.parser.peek() == Some(b'}') {
            return Ok(None);
        }
        if !self.first {
            match self.parser.bump() {
                Some(b',') => self.parser.skip_whitespace(),
                None => return Err(self.parser.error_eof("unexpected end of input in object")),
                _ => return Err(self.parser.error("expected `,` or `}` in object")),
            }
        }
        self.first = false;
        match self.parser.peek() {
            Some(b'"') => {}
            None => return Err(self.parser.error_eof("unexpected end of input in object")),
            _ => return Err(self.parser.error("expected string key in object")),
        }
        let key = self.parser.parse_string()?;
        seed.deserialize(MapKeyDeserializer(key)).map(Some)
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Error>
    where
        T: DeserializeSeed<'de>,
    {
        self.parser.skip_whitespace();
        match self.parser.bump() {
            Some(b':') => {}
            None => return Err(self.parser.error_eof("unexpected end of input in object")),
            _ => return Err(self.parser.error("expected `:` after object key")),
        }
        seed.deserialize(&mut *self.parser)
    }
}

struct EnumDeserializer<'a, 'de> {
    parser: &'a mut Parser<'de>,
    variant: String,
}

impl<'de, 'a> EnumAccess<'de> for EnumDeserializer<'a, 'de> {
    type Error = Error;
    type Variant = VariantDeserializer<'a, 'de>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = seed.deserialize(self.variant.into_deserializer())?;
        Ok((
            value,
            VariantDeserializer {
                parser: self.parser,
            },
        ))
    }
}

struct VariantDeserializer<'a, 'de> {
    parser: &'a mut Parser<'de>,
}

impl<'de> VariantAccess<'de> for VariantDeserializer<'_, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        self.parser.skip_whitespace();
        consume_literal(self.parser, "null")
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.parser)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.parser.skip_whitespace();
        if self.parser.peek() != Some(b'[') {
            return Err(self.parser.error("expected array for tuple variant"));
        }
        parse_seq(self.parser, visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.parser.skip_whitespace();
        if self.parser.peek() != Some(b'{') {
            return Err(self.parser.error("expected object for struct variant"));
        }
        parse_map(self.parser, visitor)
    }
}

/// A restricted deserializer used only for map keys, since JSON object keys
/// are always strings. Numeric/bool target types get their value parsed
/// from the key text (matching `serde_json`'s `MapKeyDeserializer`); other
/// target types just get the raw string.
struct MapKeyDeserializer(String);

macro_rules! deserialize_key_number {
    ($($method:ident => $visit:ident : $ty:ty),* $(,)?) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value, Error>
            where
                V: Visitor<'de>,
            {
                match self.0.parse::<$ty>() {
                    Ok(n) => visitor.$visit(n),
                    Err(_) => Err(<Error as de::Error>::custom(alloc::format!(
                        "invalid map key: expected {}",
                        stringify!($ty)
                    ))),
                }
            }
        )*
    };
}

impl<'de> de::Deserializer<'de> for MapKeyDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.0)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.0)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.0)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.0.as_str() {
            "true" => visitor.visit_bool(true),
            "false" => visitor.visit_bool(false),
            _ => Err(<Error as de::Error>::custom(
                "invalid map key: expected `true` or `false`",
            )),
        }
    }

    deserialize_key_number! {
        deserialize_i8 => visit_i8: i8,
        deserialize_i16 => visit_i16: i16,
        deserialize_i32 => visit_i32: i32,
        deserialize_i64 => visit_i64: i64,
        deserialize_u8 => visit_u8: u8,
        deserialize_u16 => visit_u16: u16,
        deserialize_u32 => visit_u32: u32,
        deserialize_u64 => visit_u64: u64,
        deserialize_f32 => visit_f32: f32,
        deserialize_f64 => visit_f64: f64,
    }

    serde::forward_to_deserialize_any! {
        i128 u128 char bytes byte_buf option unit unit_struct newtype_struct
        seq tuple tuple_struct map struct enum identifier ignored_any
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Map;
    use alloc::collections::BTreeMap;
    use alloc::string::String as AString;
    use alloc::vec::Vec;
    use serde::Deserialize;

    #[test]
    fn deserializes_value() {
        let v: Value = from_str(r#"{"a":[1,2,true]}"#).unwrap();
        let mut expected = Map::new();
        expected.insert(
            AString::from("a"),
            Value::Array(alloc::vec![
                Value::Number(crate::Number::from(1u64)),
                Value::Number(crate::Number::from(2u64)),
                Value::Bool(true),
            ]),
        );
        assert_eq!(v, Value::Object(expected));
    }

    #[test]
    fn rejects_trailing_data() {
        assert!(from_str::<Value>("null garbage").is_err());
    }

    #[test]
    fn deserializes_scalars() {
        assert!(from_str::<bool>("true").unwrap());
        assert_eq!(from_str::<i32>("-5").unwrap(), -5);
        assert_eq!(from_str::<u64>("42").unwrap(), 42);
        assert_eq!(from_str::<f64>("1.5").unwrap(), 1.5);
        assert_eq!(from_str::<AString>(r#""hi""#).unwrap(), "hi");
        assert_eq!(from_str::<char>(r#""x""#).unwrap(), 'x');
    }

    #[test]
    fn deserializes_option() {
        assert_eq!(from_str::<Option<i32>>("null").unwrap(), None);
        assert_eq!(from_str::<Option<i32>>("5").unwrap(), Some(5));
    }

    #[test]
    fn deserializes_collections() {
        assert_eq!(
            from_str::<Vec<i32>>("[1,2,3]").unwrap(),
            alloc::vec![1, 2, 3]
        );
        let map: BTreeMap<AString, i32> = from_str(r#"{"a":1,"b":2}"#).unwrap();
        let mut expected = BTreeMap::new();
        expected.insert(AString::from("a"), 1);
        expected.insert(AString::from("b"), 2);
        assert_eq!(map, expected);
    }

    #[test]
    fn deserializes_integer_keyed_map() {
        let map: BTreeMap<i32, AString> = from_str(r#"{"1":"a","2":"b"}"#).unwrap();
        let mut expected = BTreeMap::new();
        expected.insert(1, AString::from("a"));
        expected.insert(2, AString::from("b"));
        assert_eq!(map, expected);
    }

    #[test]
    fn rejects_invalid_integer_key() {
        assert!(from_str::<BTreeMap<i32, i32>>(r#"{"x":1}"#).is_err());
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[test]
    fn deserializes_derived_struct() {
        let p: Point = from_str(r#"{"x":1,"y":-2}"#).unwrap();
        assert_eq!(p, Point { x: 1, y: -2 });
    }

    #[derive(Deserialize, PartialEq, Debug)]
    enum Shape {
        Unit,
        Newtype(i32),
        Tuple(i32, i32),
        Struct { w: i32, h: i32 },
    }

    #[test]
    fn deserializes_derived_enum_variants() {
        assert_eq!(from_str::<Shape>(r#""Unit""#).unwrap(), Shape::Unit);
        assert_eq!(
            from_str::<Shape>(r#"{"Newtype":5}"#).unwrap(),
            Shape::Newtype(5)
        );
        assert_eq!(
            from_str::<Shape>(r#"{"Tuple":[1,2]}"#).unwrap(),
            Shape::Tuple(1, 2)
        );
        assert_eq!(
            from_str::<Shape>(r#"{"Struct":{"w":3,"h":4}}"#).unwrap(),
            Shape::Struct { w: 3, h: 4 }
        );
    }

    #[test]
    fn round_trips_derived_types_through_serde_json() {
        let p = Point { x: 7, y: -8 };
        let json = serde_json::to_string(&serde_json::json!({"x": 7, "y": -8})).unwrap();
        assert_eq!(from_str::<Point>(&json).unwrap(), p);
    }

    #[test]
    fn matches_serde_json_error_behavior_on_malformed_input() {
        assert!(from_str::<Point>(r#"{"x":1}"#).is_err()); // missing field
        assert!(from_str::<i32>(r#""not a number""#).is_err());
    }

    #[test]
    fn from_slice_matches_from_str() {
        let v: Value = from_slice(br#"{"a":1}"#).unwrap();
        assert_eq!(v, from_str::<Value>(r#"{"a":1}"#).unwrap());
    }
}
