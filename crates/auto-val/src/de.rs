//! serde `Deserializer` for the static-config subset of [`Value`](crate::Value).
//!
//! Behind the `serde` feature. Lets any `#[derive(Deserialize)]` struct read
//! directly from a parsed `Value` (or a `Node`'s body), replacing hand-written
//! `opt_string`/`opt_uint` field extraction.
//!
//! ```ignore
//! # #[cfg(feature = "serde")] {
//! use auto_val::{Value, Obj};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct Role { name: String, tier: Option<String>, skills: Vec<String> }
//!
//! // build a Value::Obj (or parse one from .at via AtomParser)
//! let mut o = Obj::new();
//! o.set("name", Value::str("coder"));
//! o.set("tier", Value::str("max"));
//! o.set("skills", Value::array_of(["tdd", "review"]));
//! let v = Value::Obj(o);
//!
//! let role: Role = v.deserialize_into().unwrap();
//! assert_eq!(role.name, "coder");
//! assert_eq!(role.tier.as_deref(), Some("max"));
//! assert_eq!(role.skills, vec!["tdd".to_string(), "review".to_string()]);
//! # }
//! ```
//!
//! ## Scope (Plan 381)
//!
//! Covers the static-config subset a parsed `.at` produces: `Str`, the integer
//! variants, `Double`/`Float`, `Bool`, `Nil`/`Null`/`Void` (→ unit / Option
//! None), `Array`/`Block` (→ seq), and `Obj` (→ map). VM/runtime variants
//! (`Fn`, `Closure`, `Widget`, `Grid`, `VmRef`, …) cannot be deserialized and
//! return an error.
//!
//! Why a hand-written Deserializer rather than `#[derive(Deserialize)]` on
//! `Value`: `AutoStr = ecow::EcoString` (no serde impl), `Value` carries ~50
//! VM-only variants, and `Obj` keys are `ValueKey` (not `String`). The adapter
//! sidesteps all three — see Plan 381 §2.1.

use crate::{Value, ValueKey};
use serde::de::{
    self, Deserialize, DeserializeSeed, Error as DeErrorTrait, MapAccess, SeqAccess, Visitor,
};
use serde::de::value::Error as DeError;
use std::marker::PhantomData;

/// A wrapper that makes `&Value` usable as a `serde::de::Deserializer`.
///
/// Construct via [`Value::deserialize_into`] or directly:
///
/// ```ignore
/// # #[cfg(feature = "serde")] {
/// let v = auto_val::Value::str("hi");
/// let s: String = serde::Deserialize::deserialize(auto_val::ValueDeserializer(&v)).unwrap();
/// assert_eq!(s, "hi");
/// # }
/// ```
pub struct ValueDeserializer<'a>(pub &'a Value);

impl Value {
    /// Deserialize a `T` from this value.
    ///
    /// ```ignore
    /// # #[cfg(feature = "serde")] {
    /// let v = auto_val::Value::Int(7);
    /// let n: i32 = v.deserialize_into().unwrap();
    /// assert_eq!(n, 7);
    /// # }
    /// ```
    pub fn deserialize_into<'de, T: Deserialize<'de>>(&'de self) -> Result<T, DeError> {
        T::deserialize(ValueDeserializer(self))
    }
}

/// Extension adding a serde entry point on [`Node`](crate::Node).
///
/// `Node::deserialize` treats the node's **props** as a map (v1: kids are not
/// included — nested named blocks need a field-level resolver, future work).
/// This matches how config structs read their fields: `role { name: …, tier: … }`
/// → `RoleDecl { name, tier }`.
impl crate::Node {
    /// Deserialize a `T` from this node's props (Plan 381 Phase B).
    ///
    /// ```ignore
    /// # #[cfg(feature = "serde")] {
    /// use auto_atom::AtomParser;
    /// use serde::Deserialize;
    /// #[derive(Deserialize)] struct Role { name: String }
    /// let atom = AtomParser::parse("role { name : \"coder\" }").unwrap();
    /// if let auto_atom::Atom::Node(n) = atom {
    ///     let r: Role = n.deserialize().unwrap();
    ///     assert_eq!(r.name, "coder");
    /// }
    /// # }
    /// ```
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self) -> Result<T, DeError> {
        // Reuse the Value::Node branch in deserialize_any, which walks props.
        // The owned Value must outlive the deserialization; bind it locally.
        // Using a HRTB on T avoids tying the output lifetime to &self.
        let as_value = Value::Node(self.clone());
        T::deserialize(ValueDeserializer(&as_value))
    }
}

impl<'de> de::Deserializer<'de> for ValueDeserializer<'de> {
    type Error = DeError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, DeError> {
        match self.0 {
            // strings
            Value::Str(s) => visitor.visit_str(s.as_str()),
            Value::String(s) => visitor.visit_str(s.as_str()),
            // StrSlice::as_str is unsafe (may be borrowed/unterminated); use Display.
            Value::StrSlice(s) => visitor.visit_str(&format!("{s}")),
            Value::CStr(s) => visitor.visit_str(&format!("{s}")),
            Value::Char(c) => visitor.visit_char(*c),
            // integers
            Value::Int(i) => visitor.visit_i32(*i),
            Value::I8(i) => visitor.visit_i8(*i),
            Value::U8(u) | Value::Byte(u) => visitor.visit_u8(*u),
            Value::Uint(u) => visitor.visit_u64(*u as u64),
            Value::USize(u) => visitor.visit_u64(*u as u64),
            Value::I64(i) => visitor.visit_i64(*i),
            // floats
            Value::Float(f) | Value::Double(f) => visitor.visit_f64(*f),
            // bool
            Value::Bool(b) => visitor.visit_bool(*b),
            // nullish → unit (covers Option<T> when the field is absent OR null)
            Value::Nil | Value::Null | Value::Void | Value::None => visitor.visit_unit(),
            // Option<Some>
            Value::Some(inner) => ValueDeserializer(inner).deserialize_any(visitor),
            // Result
            Value::Ok(inner) => ValueDeserializer(inner).deserialize_any(visitor),
            Value::Err(msg) => Err(DeErrorTrait::custom(format!("Err value: {msg}"))),
            // sequences
            Value::Array(a) | Value::Block(a) => {
                visitor.visit_seq(SeqAccess_ { iter: a.values.iter(), _life: PhantomData })
            }
            // map
            Value::Obj(o) => {
                visitor.visit_map(MapAccess_ { iter: o.iter(), current_value: None, _life: PhantomData })
            }
            // Node: treat its body (props) as a map. This keeps Node usable as
            // a deserialize target without callers having to extract props.
            Value::Node(n) => {
                // Synthesize an Obj view from the node's props for the visitor.
                // We can't borrow n.props directly across the visit_map call
                // cleanly, so walk props via a small adapter: build a temporary
                // iterator over (&ValueKey, &Value).
                let props: Vec<(&ValueKey, &Value)> = n.props_iter().collect();
                visitor.visit_map(NodePropsMap {
                    props: props.into_iter(),
                    current_value: None,
                    _life: PhantomData,
                })
            }
            // everything else is a VM/runtime value — not deserializable
            other => Err(DeErrorTrait::custom(format!(
                "cannot deserialize Value variant `{other:?}` (VM/runtime type, not a config value)"
            ))),
        }
    }

    // For the typed entry points, defer to deserialize_any. serde's derive
    // calls deserialize_any for fields whose Rust type is generic/enum, and the
    // specific deserialize_* for known types; routing them all through
    // deserialize_any is correct here because the Value carries its own type.

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, DeError> {
        // Absent-in-Obj is handled by MapAccess (next_key_seed returns None);
        // here we only see a present value. Nullish → None, else Some.
        match self.0 {
            Value::Nil | Value::Null | Value::Void | Value::None => visitor.visit_none(),
            _ => visitor.visit_some(ValueDeserializer(self.0)),
        }
    }

    // For a serde-derived enum, the value is the variant name (a string for the
    // common externally-tagged representation). Hand the name to visit_enum.
    // Struct/struct-variant enums are rarer in config; if the value isn't a
    // string, fall back to deserialize_any (which may still work for unit-style).
    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, DeError> {
        match self.0 {
            Value::Str(s) => visitor.visit_enum(StrEnumAccess(s.as_str())),
            _ => self.deserialize_any(visitor),
        }
    }

    // Forward the remaining required methods to deserialize_any. ( serde allows
    // implementing just deserialize_any + deserialize_option; the rest get a
    // default that calls deserialize_any via &self. We implement them explicitly
    // only where behavior differs — option + enum, above. )
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf unit unit_struct newtype_struct seq tuple tuple_struct map
        struct identifier ignored_any
    }
}

/// Enum access where the variant is a bare string (externally-tagged enum).
struct StrEnumAccess<'a>(&'a str);
impl<'de> de::EnumAccess<'de> for StrEnumAccess<'de> {
    type Error = DeError;
    type Variant = UnitVariantAccess;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant), DeError> {
        let v = seed.deserialize(StrDeserializer(self.0))?;
        Ok((v, UnitVariantAccess))
    }
}

/// A variant with no further data (unit variant). Externally-tagged string
/// enums like `Kind::File` carry no payload.
struct UnitVariantAccess;
impl<'de> de::VariantAccess<'de> for UnitVariantAccess {
    type Error = DeError;
    fn unit_variant(self) -> Result<(), DeError> {
        Ok(())
    }
    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, _seed: T) -> Result<T::Value, DeError> {
        Err(DeErrorTrait::custom("expected unit variant, found newtype"))
    }
    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, _visitor: V) -> Result<V::Value, DeError> {
        Err(DeErrorTrait::custom("expected unit variant, found tuple"))
    }
    fn struct_variant<V: Visitor<'de>>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value, DeError> {
        Err(DeErrorTrait::custom("expected unit variant, found struct"))
    }
}

struct SeqAccess_<'a> {
    iter: std::slice::Iter<'a, Value>,
    _life: PhantomData<&'a ()>,
}

impl<'de> SeqAccess<'de> for SeqAccess_<'de> {
    type Error = DeError;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, DeError> {
        match self.iter.next() {
            Some(v) => seed.deserialize(ValueDeserializer(v)).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

// ---- MapAccess over Obj (key = ValueKey) ----

struct MapAccess_<'a> {
    iter: indexmap::map::Iter<'a, ValueKey, Value>,
    current_value: Option<&'a Value>,
    _life: PhantomData<&'a ()>,
}

impl<'de> MapAccess<'de> for MapAccess_<'de> {
    type Error = DeError;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, DeError> {
        match self.iter.next() {
            Some((k, v)) => {
                // serde map keys deserialize from strings; ValueKey::Str holds
                // the name. Int/Bool keys are exotic in config → error.
                let key_str = k.name().ok_or_else(|| {
                    DeErrorTrait::custom("non-string Obj key cannot be deserialized into a map")
                })?;
                self.current_value = Some(v);
                seed.deserialize(StrDeserializer(key_str)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, DeError> {
        let v = self
            .current_value
            .take()
            .expect("next_value_seed called before next_key_seed");
        seed.deserialize(ValueDeserializer(v))
    }
}

// ---- MapAccess over a Node's props (for Value::Node) ----

struct NodePropsMap<'a> {
    props: std::vec::IntoIter<(&'a ValueKey, &'a Value)>,
    current_value: Option<&'a Value>,
    _life: PhantomData<&'a ()>,
}

impl<'de> MapAccess<'de> for NodePropsMap<'de> {
    type Error = DeError;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, DeError> {
        match self.props.next() {
            Some((k, v)) => {
                let key_str = k.name().ok_or_else(|| {
                    DeErrorTrait::custom("non-string Node prop key cannot be deserialized")
                })?;
                self.current_value = Some(v);
                seed.deserialize(StrDeserializer(key_str)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, DeError> {
        let v = self
            .current_value
            .take()
            .expect("next_value_seed called before next_key_seed");
        seed.deserialize(ValueDeserializer(v))
    }
}

// ---- a tiny Deserializer that always yields one string (for map keys) ----

struct StrDeserializer<'a>(&'a str);

impl<'de> de::Deserializer<'de> for StrDeserializer<'de> {
    type Error = DeError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, DeError> {
        visitor.visit_str(self.0)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Array, Obj};
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Scalars {
        name: String,
        count: i32,
        rate: f64,
        enabled: bool,
    }

    fn obj() -> Value {
        let mut o = Obj::new();
        o.set("name", Value::str("demo"));
        o.set("count", Value::Int(42));
        o.set("rate", Value::Double(1.5));
        o.set("enabled", Value::Bool(true));
        Value::Obj(o)
    }

    #[test]
    fn scalars_from_obj() {
        let s: Scalars = obj().deserialize_into().unwrap();
        assert_eq!(
            s,
            Scalars { name: "demo".into(), count: 42, rate: 1.5, enabled: true }
        );
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct WithOption {
        name: String,
        tier: Option<String>,
        budget: Option<i32>,
    }

    #[test]
    fn option_present_and_absent() {
        let mut o = Obj::new();
        o.set("name", Value::str("coder"));
        o.set("tier", Value::str("max"));
        // budget absent
        let r: WithOption = Value::Obj(o).deserialize_into().unwrap();
        assert_eq!(r.tier.as_deref(), Some("max"));
        assert!(r.budget.is_none());
    }

    #[test]
    fn option_null_becomes_none() {
        let mut o = Obj::new();
        o.set("name", Value::str("x"));
        o.set("tier", Value::Nil); // explicit null
        let r: WithOption = Value::Obj(o).deserialize_into().unwrap();
        assert!(r.tier.is_none());
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct WithVec {
        skills: Vec<String>,
    }

    #[test]
    fn vec_of_strings_from_array() {
        let mut o = Obj::new();
        o.set(
            "skills",
            Value::Array(Array {
                values: vec![Value::str("tdd"), Value::str("review")],
            }),
        );
        let r: WithVec = Value::Obj(o).deserialize_into().unwrap();
        assert_eq!(r.skills, vec!["tdd".to_string(), "review".to_string()]);
    }

    #[test]
    fn empty_array_becomes_empty_vec() {
        let mut o = Obj::new();
        o.set("skills", Value::Array(Array { values: vec![] }));
        let r: WithVec = Value::Obj(o).deserialize_into().unwrap();
        assert!(r.skills.is_empty());
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Nested {
        name: String,
        inner: Inner,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Inner {
        key: String,
    }

    #[test]
    fn nested_obj() {
        let mut inner = Obj::new();
        inner.set("key", Value::str("v"));
        let mut o = Obj::new();
        o.set("name", Value::str("outer"));
        o.set("inner", Value::Obj(inner));
        let r: Nested = Value::Obj(o).deserialize_into().unwrap();
        assert_eq!(r.inner.key, "v");
    }

    #[test]
    fn type_mismatch_is_error() {
        let mut o = Obj::new();
        o.set("name", Value::Int(7)); // expected String
        let r: Result<WithOption, _> = Value::Obj(o).deserialize_into();
        assert!(r.is_err());
    }

    #[test]
    fn missing_required_field_is_error() {
        let mut o = Obj::new();
        // name required but absent
        o.set("tier", Value::str("max"));
        let r: Result<WithOption, _> = Value::Obj(o).deserialize_into();
        assert!(r.is_err());
    }

    #[test]
    fn top_level_scalar() {
        let v = Value::str("hello");
        let s: String = v.deserialize_into().unwrap();
        assert_eq!(s, "hello");

        let v = Value::Int(-3);
        let n: i32 = v.deserialize_into().unwrap();
        assert_eq!(n, -3);
    }

    #[test]
    fn top_level_vec() {
        let v = Value::Array(Array {
            values: vec![Value::Int(1), Value::Int(2), Value::Int(3)],
        });
        let ns: Vec<i32> = v.deserialize_into().unwrap();
        assert_eq!(ns, vec![1, 2, 3]);
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Uints {
        a: u32,
        b: u64,
    }

    #[test]
    fn uint_variants() {
        let mut o = Obj::new();
        o.set("a", Value::Uint(7));
        o.set("b", Value::USize(99));
        let r: Uints = Value::Obj(o).deserialize_into().unwrap();
        assert_eq!(r, Uints { a: 7, b: 99 });
    }

    #[test]
    fn enum_string_field() {
        // serde enum with rename_all; bare idents parse to Value::Str so this
        // mirrors reading `kind : file` from a .at.
        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(rename_all = "lowercase")]
        enum Kind {
            File,
            Collection,
            Custom,
        }
        #[derive(Debug, Deserialize, PartialEq)]
        struct Decl {
            kind: Kind,
        }
        let mut o = Obj::new();
        o.set("kind", Value::str("file"));
        let r: Decl = Value::Obj(o).deserialize_into().unwrap();
        assert_eq!(r.kind, Kind::File);
    }

    #[test]
    fn vm_variant_is_error() {
        // Lambda is a VM type — not deserializable.
        let v = Value::Lambda("lambda_id".into());
        let r: Result<String, _> = v.deserialize_into();
        assert!(r.is_err());
    }

    #[test]
    fn node_deserialize_reads_props() {
        // Mirrors reading `role { name: "coder", tier: "max" }` after parsing:
        // the Node carries name/tier as props; Node::deserialize treats props as
        // a map (v1: kids excluded).
        use crate::Node;
        let mut node = Node::new("role");
        node.set_prop("name", Value::str("coder"));
        node.set_prop("tier", Value::str("max"));
        node.set_prop("budget", Value::Int(5000));

        #[derive(Debug, Deserialize, PartialEq)]
        struct Role {
            name: String,
            tier: String,
            budget: i32,
        }
        let r: Role = node.deserialize().unwrap();
        assert_eq!(r, Role { name: "coder".into(), tier: "max".into(), budget: 5000 });
    }

    #[test]
    fn node_deserialize_option_absent() {
        use crate::Node;
        let mut node = Node::new("role");
        node.set_prop("name", Value::str("x"));
        // tier absent
        #[derive(Debug, Deserialize, PartialEq)]
        struct Role {
            name: String,
            tier: Option<String>,
        }
        let r: Role = node.deserialize().unwrap();
        assert!(r.tier.is_none());
    }
}
