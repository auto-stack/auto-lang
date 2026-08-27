//! serde `Serializer` for the static-config subset of [`Value`](crate::Value).
//!
//! Behind the `serde` feature (same gate as [`crate::de`]). Lets any
//! `#[derive(Serialize)]` struct produce a `Value` — the mirror of
//! [`crate::ValueDeserializer`] (Plan 381) — completing the chain
//! `struct → Value → Node → .at source` (the last hops via
//! [`node_from_value`] and `AtomSource::to_at_source`).
//!
//! ```ignore
//! # #[cfg(feature = "serde")] {
//! use auto_val::{node_from_value, Value};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize, PartialEq)]
//! struct Role { name: String, tier: Option<String>, skills: Vec<String> }
//!
//! let role = Role { name: "coder".into(), tier: Some("max".into()),
//!                   skills: vec!["tdd".into(), "review".into()] };
//!
//! let v = auto_val::to_value(&role).unwrap();
//! let back: Role = v.deserialize_into().unwrap();
//! assert_eq!(back, role);
//!
//! let node = node_from_value("role", &role).unwrap();
//! assert!(node.to_at_source().contains("name"));
//! # }
//! ```
//!
//! ## Scope (Plan 332 S1)
//!
//! Produces the same static-config subset [`crate::de`] reads back: scalars
//! (`Str`, integer variants, `Float`/`Double`, `Bool`, `Char`), `Nil` (unit /
//! `None`), `Array` (seq/tuple/tuple-struct), `Obj` (map/struct), and
//! externally-tagged unit enums as bare strings. Struct/tuple **variants**
//! error, mirroring `de`'s unit-only `EnumAccess`.

use crate::{Array, Node, Obj, Value};
use serde::ser::{self, Error as SerErrorTrait, Impossible, Serialize};

/// Error produced by [`ValueSerializer`]. Constructed via `serde::ser::Error`.
#[derive(Debug, Clone)]
pub struct SerError {
    msg: String,
}

impl std::fmt::Display for SerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)
    }
}

impl std::error::Error for SerError {}

impl ser::Error for SerError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        SerError {
            msg: msg.to_string(),
        }
    }
}

/// The serializer proper. `Ok = Value` — driving `T::serialize(ValueSerializer)`
/// yields the `Value` tree (see [`to_value`]).
pub struct ValueSerializer;

/// Serialize any `#[derive(Serialize)]` type into a [`Value`].
///
/// The mirror of [`Value::deserialize_into`](crate::Value::deserialize_into).
pub fn to_value<T: Serialize + ?Sized>(value: &T) -> Result<Value, SerError> {
    value.serialize(ValueSerializer)
}

impl Value {
    /// Serialize `value` into a `Value` (mirror of `deserialize_into`).
    pub fn serialize_from<T: Serialize + ?Sized>(value: &T) -> Result<Value, SerError> {
        to_value(value)
    }
}

/// Serialize `T` into a **named** node — the `.at` shape `name { … }`.
///
/// `value` must serialize to a map/struct (`Value::Obj`); its entries become
/// the node's props. This is the `#[atom(node = "role")]` container semantics:
/// serde has no notion of a root node name, so the caller supplies it here.
pub fn node_from_value<T: Serialize + ?Sized>(
    node_name: &str,
    value: &T,
) -> Result<Node, SerError> {
    match to_value(value)? {
        Value::Obj(o) => {
            let mut node = Node::new(node_name);
            for (k, v) in o.iter() {
                let key = k
                    .name()
                    .ok_or_else(|| SerError::custom("non-string key cannot become a node prop"))?;
                node.set_prop(key, v.clone());
            }
            Ok(node)
        }
        other => Err(SerError::custom(format!(
            "node_from_value expects a map/struct, got `{other:?}`"
        ))),
    }
}

impl ser::Serializer for ValueSerializer {
    type Ok = Value;
    type Error = SerError;
    type SerializeSeq = SerializeArray;
    type SerializeTuple = SerializeArray;
    type SerializeTupleStruct = SerializeArray;
    type SerializeTupleVariant = Impossible<Value, SerError>;
    type SerializeMap = SerializeObjMap;
    type SerializeStruct = SerializeObjStruct;
    type SerializeStructVariant = Impossible<Value, SerError>;

    // -- scalars (variant choice mirrors de.rs's visit_* mapping) --

    fn serialize_bool(self, v: bool) -> Result<Value, SerError> {
        Ok(Value::Bool(v))
    }
    fn serialize_i8(self, v: i8) -> Result<Value, SerError> {
        Ok(Value::I8(v))
    }
    fn serialize_i16(self, v: i16) -> Result<Value, SerError> {
        Ok(Value::Int(v as i32))
    }
    fn serialize_i32(self, v: i32) -> Result<Value, SerError> {
        Ok(Value::Int(v))
    }
    fn serialize_i64(self, v: i64) -> Result<Value, SerError> {
        Ok(Value::I64(v))
    }
    fn serialize_u8(self, v: u8) -> Result<Value, SerError> {
        Ok(Value::U8(v))
    }
    fn serialize_u16(self, v: u16) -> Result<Value, SerError> {
        Ok(Value::Uint(v as u32))
    }
    fn serialize_u32(self, v: u32) -> Result<Value, SerError> {
        Ok(Value::Uint(v))
    }
    fn serialize_u64(self, v: u64) -> Result<Value, SerError> {
        Ok(Value::USize(v as usize))
    }
    fn serialize_f32(self, v: f32) -> Result<Value, SerError> {
        Ok(Value::Float(v as f64))
    }
    fn serialize_f64(self, v: f64) -> Result<Value, SerError> {
        Ok(Value::Double(v))
    }
    fn serialize_char(self, v: char) -> Result<Value, SerError> {
        Ok(Value::Char(v))
    }
    fn serialize_str(self, v: &str) -> Result<Value, SerError> {
        Ok(Value::str(v))
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Value, SerError> {
        Ok(Value::Array(Array {
            values: v.iter().map(|b| Value::Byte(*b)).collect(),
        }))
    }

    // -- none/unit (de maps the nullish family back to unit/Option::None) --

    fn serialize_none(self) -> Result<Value, SerError> {
        Ok(Value::Nil)
    }
    fn serialize_unit(self) -> Result<Value, SerError> {
        Ok(Value::Nil)
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value, SerError> {
        Ok(Value::Nil)
    }
    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<Value, SerError> {
        value.serialize(self)
    }

    // -- enums: externally-tagged bare string, mirroring de's StrEnumAccess --

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value, SerError> {
        Ok(Value::str(variant))
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Value, SerError> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Value, SerError> {
        Err(SerError::custom(
            "newtype enum variants are not supported in the .at config subset (unit variants only)",
        ))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, SerError> {
        Err(SerError::custom(
            "tuple enum variants are not supported in the .at config subset (unit variants only)",
        ))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, SerError> {
        Err(SerError::custom(
            "struct enum variants are not supported in the .at config subset (unit variants only)",
        ))
    }

    // -- composites --

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, SerError> {
        Ok(SerializeArray {
            values: Vec::with_capacity(len.unwrap_or(0)),
        })
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, SerError> {
        Ok(SerializeArray { values: Vec::new() })
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, SerError> {
        Ok(SerializeArray { values: Vec::new() })
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, SerError> {
        Ok(SerializeObjMap {
            obj: Obj::new(),
            pending_key: None,
        })
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, SerError> {
        Ok(SerializeObjStruct { obj: Obj::new() })
    }
}

/// Seq/tuple accumulator → `Value::Array`.
pub struct SerializeArray {
    values: Vec<Value>,
}

impl ser::SerializeSeq for SerializeArray {
    type Ok = Value;
    type Error = SerError;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), SerError> {
        self.values.push(value.serialize(ValueSerializer)?);
        Ok(())
    }
    fn end(self) -> Result<Value, SerError> {
        Ok(Value::Array(Array {
            values: self.values,
        }))
    }
}

impl ser::SerializeTuple for SerializeArray {
    type Ok = Value;
    type Error = SerError;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), SerError> {
        ser::SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Value, SerError> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for SerializeArray {
    type Ok = Value;
    type Error = SerError;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), SerError> {
        ser::SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Value, SerError> {
        ser::SerializeSeq::end(self)
    }
}

/// Struct accumulator → `Value::Obj` (serde hands us `&'static str` keys).
pub struct SerializeObjStruct {
    obj: Obj,
}

impl ser::SerializeStruct for SerializeObjStruct {
    type Ok = Value;
    type Error = SerError;
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), SerError> {
        self.obj.set(key, value.serialize(ValueSerializer)?);
        Ok(())
    }
    fn end(self) -> Result<Value, SerError> {
        Ok(Value::Obj(self.obj))
    }
}

/// Map accumulator → `Value::Obj`. Keys must serialize to strings, mirroring
/// `de`'s "non-string Obj key" error.
pub struct SerializeObjMap {
    obj: Obj,
    pending_key: Option<String>,
}

impl ser::SerializeMap for SerializeObjMap {
    type Ok = Value;
    type Error = SerError;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), SerError> {
        let kv = key.serialize(ValueSerializer)?;
        let name = match kv {
            Value::Str(s) => s.to_string(),
            Value::String(s) => s.to_string(),
            other => {
                return Err(SerError::custom(format!(
                    "map key must serialize to a string, got `{other:?}`"
                )))
            }
        };
        self.pending_key = Some(name);
        Ok(())
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), SerError> {
        let key = self
            .pending_key
            .take()
            .expect("serialize_value called before serialize_key");
        self.obj.set(key, value.serialize(ValueSerializer)?);
        Ok(())
    }
    fn end(self) -> Result<Value, SerError> {
        Ok(Value::Obj(self.obj))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    // Round-trip helper: T → Value → T.
    fn rt<T>(v: &T) -> T
    where
        T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
    {
        let value = to_value(v).unwrap();
        value.deserialize_into().unwrap()
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Scalars {
        name: String,
        count: i32,
        rate: f64,
        enabled: bool,
    }

    #[test]
    fn scalars_round_trip() {
        let s = Scalars {
            name: "demo".into(),
            count: 42,
            rate: 1.5,
            enabled: true,
        };
        let v = to_value(&s).unwrap();
        // shape: Obj with typed entries
        match &v {
            Value::Obj(o) => {
                assert_eq!(o.get("name"), Some(Value::str("demo")));
                assert_eq!(o.get("count"), Some(Value::Int(42)));
                assert_eq!(o.get("rate"), Some(Value::Double(1.5)));
                assert_eq!(o.get("enabled"), Some(Value::Bool(true)));
            }
            other => panic!("expected Obj, got {other:?}"),
        }
        assert_eq!(rt(&s), s);
    }

    #[test]
    fn top_level_scalars() {
        assert_eq!(to_value("hi").unwrap(), Value::str("hi"));
        assert_eq!(to_value(&7i32).unwrap(), Value::Int(7));
        assert_eq!(to_value(&(-3i64)).unwrap(), Value::I64(-3));
        assert_eq!(to_value(&1.25f64).unwrap(), Value::Double(1.25));
        assert_eq!(to_value(&true).unwrap(), Value::Bool(true));
        assert_eq!(to_value(&'x').unwrap(), Value::Char('x'));

        let s: String = to_value("hi").unwrap().deserialize_into().unwrap();
        assert_eq!(s, "hi");
    }

    #[test]
    fn int_width_variants() {
        // Each Rust width lands in the Value variant `de` maps back losslessly.
        assert_eq!(to_value(&7u8).unwrap(), Value::U8(7));
        assert_eq!(to_value(&7u16).unwrap(), Value::Uint(7));
        assert_eq!(to_value(&7u32).unwrap(), Value::Uint(7));
        assert_eq!(to_value(&7u64).unwrap(), Value::USize(7));
        assert_eq!(to_value(&7i8).unwrap(), Value::I8(7));

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Widths {
            a: u8,
            b: u32,
            c: u64,
            d: i8,
        }
        let w = Widths {
            a: 1,
            b: 2,
            c: 3,
            d: -4,
        };
        assert_eq!(rt(&w), w);
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct WithOption {
        name: String,
        tier: Option<String>,
        budget: Option<i32>,
    }

    #[test]
    fn option_some_and_none() {
        let s = WithOption {
            name: "coder".into(),
            tier: Some("max".into()),
            budget: None,
        };
        let v = to_value(&s).unwrap();
        match &v {
            Value::Obj(o) => {
                assert_eq!(o.get("tier"), Some(Value::str("max")));
                // None serializes as Nil — de maps Nil back to Option::None.
                assert_eq!(o.get("budget"), Some(Value::Nil));
            }
            other => panic!("expected Obj, got {other:?}"),
        }
        assert_eq!(rt(&s), s);
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct WithVec {
        skills: Vec<String>,
    }

    #[test]
    fn vec_to_array_round_trip() {
        let s = WithVec {
            skills: vec!["tdd".into(), "review".into()],
        };
        let v = to_value(&s).unwrap();
        match &v {
            Value::Obj(o) => {
                assert_eq!(
                    o.get("skills"),
                    Some(Value::Array(Array {
                        values: vec![Value::str("tdd"), Value::str("review")]
                    }))
                );
            }
            other => panic!("expected Obj, got {other:?}"),
        }
        assert_eq!(rt(&s), s);
    }

    #[test]
    fn empty_vec_round_trip() {
        let s = WithVec { skills: vec![] };
        assert_eq!(rt(&s), s);
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Inner {
        key: String,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Nested {
        name: String,
        inner: Inner,
    }

    #[test]
    fn nested_struct_round_trip() {
        let s = Nested {
            name: "outer".into(),
            inner: Inner { key: "v".into() },
        };
        assert_eq!(rt(&s), s);
    }

    #[test]
    fn tuple_round_trip() {
        let t = (1i32, "two".to_string(), 3.5f64);
        let v = to_value(&t).unwrap();
        assert!(matches!(v, Value::Array(_)));
        assert_eq!(rt(&t), t);
    }

    #[test]
    fn bytes_round_trip() {
        // serde serializes plain &[u8] as a seq of u8 (no bytes specialization),
        // so it lands as Array of U8. Both U8 and Byte read back via visit_u8.
        let bytes: &[u8] = &[1, 2, 3];
        let v = to_value(&bytes).unwrap();
        assert_eq!(
            v,
            Value::Array(Array {
                values: vec![Value::U8(1), Value::U8(2), Value::U8(3)]
            })
        );
        let back: Vec<u8> = v.deserialize_into().unwrap();
        assert_eq!(back, vec![1, 2, 3]);

        // Explicit serialize_bytes callers land as Array of Byte (also reads back).
        struct RawBytes<'a>(&'a [u8]);
        impl Serialize for RawBytes<'_> {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.serialize_bytes(self.0)
            }
        }
        let v = to_value(&RawBytes(&[1, 2, 3])).unwrap();
        assert_eq!(
            v,
            Value::Array(Array {
                values: vec![Value::Byte(1), Value::Byte(2), Value::Byte(3)]
            })
        );
        let back: Vec<u8> = v.deserialize_into().unwrap();
        assert_eq!(back, vec![1, 2, 3]);
    }

    #[test]
    fn map_round_trip() {
        use std::collections::BTreeMap;
        let mut m = BTreeMap::new();
        m.insert("a".to_string(), 1i32);
        m.insert("b".to_string(), 2);
        let v = to_value(&m).unwrap();
        assert!(matches!(v, Value::Obj(_)));
        assert_eq!(rt(&m), m);
    }

    #[test]
    fn enum_unit_variant_bare_string() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(rename_all = "lowercase")]
        enum Kind {
            File,
            Collection,
        }
        assert_eq!(to_value(&Kind::File).unwrap(), Value::str("file"));
        assert_eq!(rt(&Kind::Collection), Kind::Collection);
    }

    #[test]
    fn struct_variant_is_error() {
        #[derive(Debug, Serialize)]
        enum Bad {
            Struct { x: i32 },
        }
        assert!(to_value(&Bad::Struct { x: 1 }).is_err());
    }

    #[test]
    fn non_string_map_key_is_error() {
        use std::collections::HashMap;
        let mut m: HashMap<i32, i32> = HashMap::new();
        m.insert(1, 2);
        assert!(to_value(&m).is_err());
    }

    // ---- node_from_value: the .at `name { … }` container ----

    #[test]
    fn node_from_value_builds_named_node() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Role {
            name: String,
            tier: String,
            budget: i32,
        }
        let role = Role {
            name: "coder".into(),
            tier: "max".into(),
            budget: 5000,
        };
        let node = node_from_value("role", &role).unwrap();
        assert_eq!(node.get_prop_of("name"), Value::str("coder"));
        assert_eq!(node.get_prop_of("tier"), Value::str("max"));
        assert_eq!(node.get_prop_of("budget"), Value::Int(5000));
        // full loop back through Node::deserialize (Plan 381)
        let back: Role = node.deserialize().unwrap();
        assert_eq!(back, role);
    }

    #[test]
    fn node_from_value_emits_at_source() {
        #[derive(Debug, Serialize)]
        struct Role {
            name: String,
            tier: String,
        }
        let role = Role {
            name: "precise-coder".into(),
            tier: "max".into(),
        };
        let node = node_from_value("role", &role).unwrap();
        let src = crate::AtomSource::to_at_source(&node);
        assert!(
            src.starts_with("role"),
            "source should open with the node name: {src}"
        );
        assert!(src.contains("precise-coder"), "missing field in: {src}");
        assert!(src.contains("max"), "missing field in: {src}");
    }

    #[test]
    fn node_from_value_rejects_non_map() {
        assert!(node_from_value("role", &42i32).is_err());
        assert!(node_from_value("role", &"str").is_err());
    }

    #[test]
    fn value_serialize_from_alias() {
        assert_eq!(Value::serialize_from(&7i32).unwrap(), Value::Int(7));
    }
}
