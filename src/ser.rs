use crate::{Error, Number};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use serde::ser::{
    self, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};

/// Serializes any `Serialize` value to a compact JSON string.
pub fn to_string<T>(value: &T) -> Result<String, Error>
where
    T: Serialize + ?Sized,
{
    let mut serializer = Serializer {
        output: String::new(),
        pretty: false,
        indent: 0,
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

/// Serializes any `Serialize` value to a pretty-printed JSON string,
/// indented two spaces per level. Empty arrays/objects are rendered inline
/// (`[]`, `{}`) rather than spread across lines.
pub fn to_string_pretty<T>(value: &T) -> Result<String, Error>
where
    T: Serialize + ?Sized,
{
    let mut serializer = Serializer {
        output: String::new(),
        pretty: true,
        indent: 0,
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

/// Serializes any `Serialize` value to a compact JSON byte vector.
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: Serialize + ?Sized,
{
    to_string(value).map(String::into_bytes)
}

/// Serializes any `Serialize` value to a pretty-printed JSON byte vector.
pub fn to_vec_pretty<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: Serialize + ?Sized,
{
    to_string_pretty(value).map(String::into_bytes)
}

/// Serializes any `Serialize` value as compact JSON directly to a
/// [`std::io::Write`] sink.
#[cfg(feature = "std")]
pub fn to_writer<W, T>(mut writer: W, value: &T) -> Result<(), Error>
where
    W: std::io::Write,
    T: Serialize + ?Sized,
{
    writer.write_all(to_string(value)?.as_bytes())?;
    Ok(())
}

/// Serializes any `Serialize` value as pretty-printed JSON directly to a
/// [`std::io::Write`] sink.
#[cfg(feature = "std")]
pub fn to_writer_pretty<W, T>(mut writer: W, value: &T) -> Result<(), Error>
where
    W: std::io::Write,
    T: Serialize + ?Sized,
{
    writer.write_all(to_string_pretty(value)?.as_bytes())?;
    Ok(())
}

struct Serializer {
    output: String,
    pretty: bool,
    indent: usize,
}

impl Serializer {
    fn newline_and_indent(&mut self) {
        if self.pretty {
            self.output.push('\n');
            for _ in 0..self.indent {
                self.output.push_str("  ");
            }
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

/// Serializes a JSON number, delegating non-finite floats to `null` (JSON
/// has no way to represent them), same as `Value::from(f64)`.
fn write_number(n: Number, out: &mut String) {
    out.push_str(&n.to_string());
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Compound<'a>;
    type SerializeTuple = Compound<'a>;
    type SerializeTupleStruct = Compound<'a>;
    type SerializeTupleVariant = Compound<'a>;
    type SerializeMap = Compound<'a>;
    type SerializeStruct = Compound<'a>;
    type SerializeStructVariant = Compound<'a>;

    fn serialize_bool(self, v: bool) -> Result<(), Error> {
        self.output.push_str(if v { "true" } else { "false" });
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<(), Error> {
        self.serialize_i64(i64::from(v))
    }
    fn serialize_i16(self, v: i16) -> Result<(), Error> {
        self.serialize_i64(i64::from(v))
    }
    fn serialize_i32(self, v: i32) -> Result<(), Error> {
        self.serialize_i64(i64::from(v))
    }
    fn serialize_i64(self, v: i64) -> Result<(), Error> {
        write_number(Number::from(v), &mut self.output);
        Ok(())
    }
    fn serialize_i128(self, v: i128) -> Result<(), Error> {
        match Number::from_i128(v) {
            Some(n) => {
                write_number(n, &mut self.output);
                Ok(())
            }
            None => Err(<Error as ser::Error>::custom("i128 value out of range")),
        }
    }

    fn serialize_u8(self, v: u8) -> Result<(), Error> {
        self.serialize_u64(u64::from(v))
    }
    fn serialize_u16(self, v: u16) -> Result<(), Error> {
        self.serialize_u64(u64::from(v))
    }
    fn serialize_u32(self, v: u32) -> Result<(), Error> {
        self.serialize_u64(u64::from(v))
    }
    fn serialize_u64(self, v: u64) -> Result<(), Error> {
        write_number(Number::from(v), &mut self.output);
        Ok(())
    }
    fn serialize_u128(self, v: u128) -> Result<(), Error> {
        match Number::from_u128(v) {
            Some(n) => {
                write_number(n, &mut self.output);
                Ok(())
            }
            None => Err(<Error as ser::Error>::custom("u128 value out of range")),
        }
    }

    fn serialize_f32(self, v: f32) -> Result<(), Error> {
        self.serialize_f64(f64::from(v))
    }
    fn serialize_f64(self, v: f64) -> Result<(), Error> {
        match Number::from_f64(v) {
            Some(n) => write_number(n, &mut self.output),
            None => self.output.push_str("null"),
        }
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<(), Error> {
        let mut buf = [0u8; 4];
        self.serialize_str(v.encode_utf8(&mut buf))
    }

    fn serialize_str(self, v: &str) -> Result<(), Error> {
        write_escaped_string(v, &mut self.output);
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<(), Error> {
        // JSON has no byte-string type; serialize as an array of numbers,
        // same as serde_json's default (non-serde_bytes) behavior.
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            SerializeSeq::serialize_element(&mut seq, byte)?;
        }
        SerializeSeq::end(seq)
    }

    fn serialize_none(self) -> Result<(), Error> {
        self.output.push_str("null");
        Ok(())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<(), Error> {
        self.output.push_str("null");
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.output.push('{');
        write_escaped_string(variant, &mut self.output);
        self.output.push(':');
        if self.pretty {
            self.output.push(' ');
        }
        value.serialize(&mut *self)?;
        self.output.push('}');
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Compound<'a>, Error> {
        self.output.push('[');
        self.indent += 1;
        Ok(Compound {
            ser: self,
            first: true,
            wrap: None,
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Compound<'a>, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Compound<'a>, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Compound<'a>, Error> {
        self.output.push('{');
        write_escaped_string(variant, &mut self.output);
        self.output.push(':');
        if self.pretty {
            self.output.push(' ');
        }
        self.output.push('[');
        self.indent += 1;
        Ok(Compound {
            ser: self,
            first: true,
            wrap: Some(Wrap::Array),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Compound<'a>, Error> {
        self.output.push('{');
        self.indent += 1;
        Ok(Compound {
            ser: self,
            first: true,
            wrap: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Compound<'a>, Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Compound<'a>, Error> {
        self.output.push('{');
        write_escaped_string(variant, &mut self.output);
        self.output.push(':');
        if self.pretty {
            self.output.push(' ');
        }
        self.output.push('{');
        self.indent += 1;
        Ok(Compound {
            ser: self,
            first: true,
            wrap: Some(Wrap::Object),
        })
    }

    fn collect_str<T: ?Sized + core::fmt::Display>(self, value: &T) -> Result<(), Error> {
        self.serialize_str(&alloc::format!("{value}"))
    }
}

/// What an enum-variant compound wraps its inner `[...]`/`{...}` with, once
/// it closes: an extra `}` to finish the `{"variant": ...}` envelope.
enum Wrap {
    Array,
    Object,
}

/// The shared implementation behind every `Serialize{Seq,Tuple,TupleStruct,
/// TupleVariant,Map,Struct,StructVariant}` — all of them are either a
/// comma-separated `[...]` / `{...}`, optionally wrapped in an enum-variant
/// envelope.
pub(crate) struct Compound<'a> {
    ser: &'a mut Serializer,
    first: bool,
    wrap: Option<Wrap>,
}

impl Compound<'_> {
    fn write_separator(&mut self) {
        if !self.first {
            self.ser.output.push(',');
        }
        self.first = false;
        self.ser.newline_and_indent();
    }

    fn close(self, bracket: char) -> Result<(), Error> {
        self.ser.indent -= 1;
        if !self.first {
            self.ser.newline_and_indent();
        }
        self.ser.output.push(bracket);
        match self.wrap {
            Some(Wrap::Array) | Some(Wrap::Object) => self.ser.output.push('}'),
            None => {}
        }
        Ok(())
    }
}

impl SerializeSeq for Compound<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.write_separator();
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<(), Error> {
        self.close(']')
    }
}

impl SerializeTuple for Compound<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<(), Error> {
        self.close(']')
    }
}

impl SerializeTupleStruct for Compound<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<(), Error> {
        self.close(']')
    }
}

impl SerializeTupleVariant for Compound<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<(), Error> {
        self.close(']')
    }
}

impl SerializeMap for Compound<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Error> {
        self.write_separator();
        let mut key_str = String::new();
        key.serialize(MapKeySerializer {
            output: &mut key_str,
        })?;
        write_escaped_string(&key_str, &mut self.ser.output);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.ser.output.push(':');
        if self.ser.pretty {
            self.ser.output.push(' ');
        }
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<(), Error> {
        self.close('}')
    }
}

impl SerializeStruct for Compound<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.write_separator();
        write_escaped_string(key, &mut self.ser.output);
        self.ser.output.push(':');
        if self.ser.pretty {
            self.ser.output.push(' ');
        }
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<(), Error> {
        self.close('}')
    }
}

impl SerializeStructVariant for Compound<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<(), Error> {
        self.close('}')
    }
}

/// A restricted `Serializer` used only for map keys, since JSON object keys
/// must be strings. Accepts string-like and primitive scalar types
/// (stringified), rejects anything else.
struct MapKeySerializer<'a> {
    output: &'a mut String,
}

impl ser::Serializer for MapKeySerializer<'_> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = ser::Impossible<(), Error>;
    type SerializeTuple = ser::Impossible<(), Error>;
    type SerializeTupleStruct = ser::Impossible<(), Error>;
    type SerializeTupleVariant = ser::Impossible<(), Error>;
    type SerializeMap = ser::Impossible<(), Error>;
    type SerializeStruct = ser::Impossible<(), Error>;
    type SerializeStructVariant = ser::Impossible<(), Error>;

    fn serialize_str(self, v: &str) -> Result<(), Error> {
        self.output.push_str(v);
        Ok(())
    }

    fn serialize_bool(self, v: bool) -> Result<(), Error> {
        self.output.push_str(if v { "true" } else { "false" });
        Ok(())
    }
    fn serialize_i8(self, v: i8) -> Result<(), Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result<(), Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }
    fn serialize_i32(self, v: i32) -> Result<(), Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result<(), Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> Result<(), Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<(), Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<(), Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<(), Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<(), Error> {
        self.output.push(v);
        Ok(())
    }

    fn serialize_f32(self, _v: f32) -> Result<(), Error> {
        Err(<Error as ser::Error>::custom("map key must be a string"))
    }
    fn serialize_f64(self, _v: f64) -> Result<(), Error> {
        Err(<Error as ser::Error>::custom("map key must be a string"))
    }
    fn serialize_bytes(self, _v: &[u8]) -> Result<(), Error> {
        Err(<Error as ser::Error>::custom("map key must be a string"))
    }
    fn serialize_none(self) -> Result<(), Error> {
        Err(<Error as ser::Error>::custom("map key must be a string"))
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), Error> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), Error> {
        Err(<Error as ser::Error>::custom("map key must be a string"))
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Error> {
        Err(<Error as ser::Error>::custom("map key must be a string"))
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Error> {
        self.output.push_str(variant);
        Ok(())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<(), Error> {
        Err(<Error as ser::Error>::custom("map key must be a string"))
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        Err(<Error as ser::Error>::custom("map key must be a string"))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Error> {
        Err(<Error as ser::Error>::custom("map key must be a string"))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        Err(<Error as ser::Error>::custom("map key must be a string"))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        Err(<Error as ser::Error>::custom("map key must be a string"))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        Err(<Error as ser::Error>::custom("map key must be a string"))
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Error> {
        Err(<Error as ser::Error>::custom("map key must be a string"))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        Err(<Error as ser::Error>::custom("map key must be a string"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_str, Map, Value};
    use alloc::string::String as AString;
    use alloc::vec::Vec;
    use serde::Serialize;

    #[test]
    fn serializes_value_scalars() {
        assert_eq!(to_string(&Value::Null).unwrap(), "null");
        assert_eq!(to_string(&Value::Bool(true)).unwrap(), "true");
        assert_eq!(to_string(&Value::Bool(false)).unwrap(), "false");
        assert_eq!(
            to_string(&Value::Number(Number::from(42u64))).unwrap(),
            "42"
        );
        assert_eq!(
            to_string(&Value::String(AString::from("hi"))).unwrap(),
            "\"hi\""
        );
    }

    #[test]
    fn escapes_special_characters() {
        assert_eq!(
            to_string(&Value::String(AString::from("a\"b\\c\nd\te"))).unwrap(),
            r#""a\"b\\c\nd\te""#
        );
        assert_eq!(
            to_string(&Value::String(AString::from("\u{0001}"))).unwrap(),
            "\"\\u0001\""
        );
    }

    #[test]
    fn serializes_array_and_object() {
        assert_eq!(to_string(&Value::Array(Vec::new())).unwrap(), "[]");
        assert_eq!(to_string(&Value::Object(Map::new())).unwrap(), "{}");

        let arr = Value::Array(alloc::vec![
            Value::Number(Number::from(1u64)),
            Value::Bool(true)
        ]);
        assert_eq!(to_string(&arr).unwrap(), "[1,true]");

        let mut map = Map::new();
        map.insert(AString::from("a"), Value::Number(Number::from(1u64)));
        assert_eq!(to_string(&Value::Object(map)).unwrap(), r#"{"a":1}"#);
    }

    #[test]
    fn pretty_prints_scalars_like_compact() {
        assert_eq!(to_string_pretty(&Value::Null).unwrap(), "null");
        assert_eq!(
            to_string_pretty(&Value::Number(Number::from(1u64))).unwrap(),
            "1"
        );
    }

    #[test]
    fn pretty_prints_empty_containers_inline() {
        assert_eq!(to_string_pretty(&Value::Array(Vec::new())).unwrap(), "[]");
        assert_eq!(to_string_pretty(&Value::Object(Map::new())).unwrap(), "{}");
    }

    #[test]
    fn pretty_prints_array_with_indentation() {
        let arr = Value::Array(alloc::vec![
            Value::Number(Number::from(1u64)),
            Value::Number(Number::from(2u64)),
        ]);
        assert_eq!(to_string_pretty(&arr).unwrap(), "[\n  1,\n  2\n]");
    }

    #[test]
    fn pretty_prints_nested_object() {
        let mut inner = Map::new();
        inner.insert(AString::from("b"), Value::Bool(true));
        let mut outer = Map::new();
        outer.insert(AString::from("a"), Value::Object(inner));
        assert_eq!(
            to_string_pretty(&Value::Object(outer)).unwrap(),
            "{\n  \"a\": {\n    \"b\": true\n  }\n}"
        );
    }

    #[test]
    fn pretty_output_round_trips() {
        let value = from_str::<Value>(r#"{"a":[1,2,{"b":null}]}"#).unwrap();
        let pretty = to_string_pretty(&value).unwrap();
        assert_eq!(from_str::<Value>(&pretty).unwrap(), value);
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
            let value = from_str::<Value>(input).unwrap();
            let serialized = to_string(&value).unwrap();
            let reparsed = from_str::<Value>(&serialized).unwrap();
            assert_eq!(value, reparsed, "round-trip failed for {input}");
        }
    }

    #[derive(Serialize)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[derive(Serialize)]
    enum Shape {
        Unit,
        Newtype(i32),
        Tuple(i32, i32),
        Struct { w: i32, h: i32 },
    }

    #[test]
    fn serializes_derived_struct() {
        let p = Point { x: 1, y: -2 };
        assert_eq!(to_string(&p).unwrap(), r#"{"x":1,"y":-2}"#);
    }

    #[test]
    fn serializes_derived_enum_variants() {
        assert_eq!(to_string(&Shape::Unit).unwrap(), r#""Unit""#);
        assert_eq!(to_string(&Shape::Newtype(5)).unwrap(), r#"{"Newtype":5}"#);
        assert_eq!(
            to_string(&Shape::Tuple(1, 2)).unwrap(),
            r#"{"Tuple":[1,2]}"#
        );
        assert_eq!(
            to_string(&Shape::Struct { w: 3, h: 4 }).unwrap(),
            r#"{"Struct":{"w":3,"h":4}}"#
        );
    }

    #[test]
    fn serializes_option_and_collections() {
        assert_eq!(to_string(&Some(5i32)).unwrap(), "5");
        assert_eq!(to_string(&Option::<i32>::None).unwrap(), "null");
        assert_eq!(to_string(&alloc::vec![1, 2, 3]).unwrap(), "[1,2,3]");
        assert_eq!(to_string(&(1, "a", true)).unwrap(), r#"[1,"a",true]"#);

        let mut map: alloc::collections::BTreeMap<AString, i32> =
            alloc::collections::BTreeMap::new();
        map.insert(AString::from("a"), 1);
        map.insert(AString::from("b"), 2);
        assert_eq!(to_string(&map).unwrap(), r#"{"a":1,"b":2}"#);
    }

    #[test]
    fn round_trips_through_serde_json_for_derived_types() {
        let p = Point { x: 7, y: 8 };
        let ours = to_string(&p).unwrap();
        let theirs = serde_json::to_string(&serde_json::json!({"x": 7, "y": 8})).unwrap();
        assert_eq!(ours, theirs);
    }

    #[test]
    fn to_vec_matches_to_string_bytes() {
        let v = Value::Array(alloc::vec![Value::Bool(true), Value::Null]);
        assert_eq!(to_vec(&v).unwrap(), to_string(&v).unwrap().into_bytes());
        assert_eq!(
            to_vec_pretty(&v).unwrap(),
            to_string_pretty(&v).unwrap().into_bytes()
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn to_writer_matches_to_string_bytes() {
        let v = Value::Array(alloc::vec![Value::Bool(true), Value::Null]);

        let mut buf = std::vec::Vec::new();
        to_writer(&mut buf, &v).unwrap();
        assert_eq!(buf, to_string(&v).unwrap().into_bytes());

        let mut buf_pretty = std::vec::Vec::new();
        to_writer_pretty(&mut buf_pretty, &v).unwrap();
        assert_eq!(buf_pretty, to_string_pretty(&v).unwrap().into_bytes());
    }

    #[cfg(feature = "std")]
    #[test]
    fn to_writer_propagates_io_errors() {
        struct FailingWriter;
        impl std::io::Write for FailingWriter {
            fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("boom"))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let err = to_writer(FailingWriter, &Value::Null).unwrap_err();
        assert!(err.is_io());
    }
}
