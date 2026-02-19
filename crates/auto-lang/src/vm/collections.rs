use super::context::VmContext;
use auto_val::{Instance, Obj, Type, Value};
use std::collections::{BTreeMap, HashMap as StdHashMap, VecDeque};
use std::any::Any;

use crate::vm::heap_object::{HeapObject, TypeTag};

// ============================================================================
// Plan 087 Phase 4: Specialized HashMap Implementation
// ============================================================================

/// Specialized HashMap variants for common type combinations
///
/// Provides compact storage for frequently used HashMap instantiations.
/// Instead of storing all values as `Value` enum (24+ bytes each),
/// we store primitive types directly (4-8 bytes each).
///
/// # Memory Benefits
/// - `HashMap<String, int>`: 4 + 24 = 28 bytes per entry vs 48 bytes (1.7x reduction)
/// - `HashMap<String, bool>`: 1 + 24 = 25 bytes per entry vs 48 bytes (1.9x reduction)
///
/// # Examples
/// - `HashMap<String, i32>` → Stores int values directly without Value wrapper
/// - `HashMap<String, bool>` → Stores bool values directly
/// - `HashMap<String, String>` → Stores String values directly
#[derive(Debug)]
pub enum SpecializedHashMap {
    /// HashMap<String, i32> - Values stored as i32
    StringInt(StdHashMap<String, i32>),
    /// HashMap<String, bool> - Values stored as bool
    StringBool(StdHashMap<String, bool>),
    /// HashMap<String, String> - Values stored as String
    StringString(StdHashMap<String, String>),
    /// HashMap<String, f64> - Values stored as f64
    StringDouble(StdHashMap<String, f64>),
    /// HashMap<String, Value> - Generic fallback
    StringValue(StdHashMap<String, Value>),
}

impl SpecializedHashMap {
    /// Create new specialized HashMap based on value type
    pub fn new(value_type: &str) -> Self {
        match value_type {
            "int" => SpecializedHashMap::StringInt(StdHashMap::new()),
            "bool" => SpecializedHashMap::StringBool(StdHashMap::new()),
            "string" => SpecializedHashMap::StringString(StdHashMap::new()),
            "double" => SpecializedHashMap::StringDouble(StdHashMap::new()),
            _ => SpecializedHashMap::StringValue(StdHashMap::new()),
        }
    }

    /// Insert a key-value pair
    pub fn insert(&mut self, key: String, value: Value) -> Result<(), String> {
        match (self, value) {
            (SpecializedHashMap::StringInt(map), Value::Int(v)) => {
                map.insert(key, v);
                Ok(())
            }
            (SpecializedHashMap::StringBool(map), Value::Bool(v)) => {
                map.insert(key, v);
                Ok(())
            }
            (SpecializedHashMap::StringString(map), Value::Str(v)) => {
                map.insert(key, v.to_string());
                Ok(())
            }
            (SpecializedHashMap::StringDouble(map), Value::Double(v)) => {
                map.insert(key, v);
                Ok(())
            }
            (SpecializedHashMap::StringValue(map), v) => {
                map.insert(key, v);
                Ok(())
            }
            _ => Err("Type mismatch in specialized HashMap insert".to_string()),
        }
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Option<Value> {
        match self {
            SpecializedHashMap::StringInt(map) => {
                map.get(key).map(|&v| Value::Int(v))
            }
            SpecializedHashMap::StringBool(map) => {
                map.get(key).map(|&v| Value::Bool(v))
            }
            SpecializedHashMap::StringString(map) => {
                map.get(key).map(|v| Value::Str(v.as_str().into()))
            }
            SpecializedHashMap::StringDouble(map) => {
                map.get(key).map(|&v| Value::Double(v))
            }
            SpecializedHashMap::StringValue(map) => {
                map.get(key).cloned()
            }
        }
    }

    /// Check if key exists
    pub fn contains_key(&self, key: &str) -> bool {
        match self {
            SpecializedHashMap::StringInt(map) => map.contains_key(key),
            SpecializedHashMap::StringBool(map) => map.contains_key(key),
            SpecializedHashMap::StringString(map) => map.contains_key(key),
            SpecializedHashMap::StringDouble(map) => map.contains_key(key),
            SpecializedHashMap::StringValue(map) => map.contains_key(key),
        }
    }

    /// Remove a key-value pair
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        match self {
            SpecializedHashMap::StringInt(map) => {
                map.remove(key).map(|v| Value::Int(v))
            }
            SpecializedHashMap::StringBool(map) => {
                map.remove(key).map(|v| Value::Bool(v))
            }
            SpecializedHashMap::StringString(map) => {
                map.remove(key).map(|v| Value::Str(v.as_str().into()))
            }
            SpecializedHashMap::StringDouble(map) => {
                map.remove(key).map(|v| Value::Double(v))
            }
            SpecializedHashMap::StringValue(map) => {
                map.remove(key)
            }
        }
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        match self {
            SpecializedHashMap::StringInt(map) => map.len(),
            SpecializedHashMap::StringBool(map) => map.len(),
            SpecializedHashMap::StringString(map) => map.len(),
            SpecializedHashMap::StringDouble(map) => map.len(),
            SpecializedHashMap::StringValue(map) => map.len(),
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        match self {
            SpecializedHashMap::StringInt(map) => map.is_empty(),
            SpecializedHashMap::StringBool(map) => map.is_empty(),
            SpecializedHashMap::StringString(map) => map.is_empty(),
            SpecializedHashMap::StringDouble(map) => map.is_empty(),
            SpecializedHashMap::StringValue(map) => map.is_empty(),
        }
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        match self {
            SpecializedHashMap::StringInt(map) => map.clear(),
            SpecializedHashMap::StringBool(map) => map.clear(),
            SpecializedHashMap::StringString(map) => map.clear(),
            SpecializedHashMap::StringDouble(map) => map.clear(),
            SpecializedHashMap::StringValue(map) => map.clear(),
        }
    }
}

impl Clone for SpecializedHashMap {
    fn clone(&self) -> Self {
        match self {
            SpecializedHashMap::StringInt(map) => SpecializedHashMap::StringInt(map.clone()),
            SpecializedHashMap::StringBool(map) => SpecializedHashMap::StringBool(map.clone()),
            SpecializedHashMap::StringString(map) => SpecializedHashMap::StringString(map.clone()),
            SpecializedHashMap::StringDouble(map) => SpecializedHashMap::StringDouble(map.clone()),
            SpecializedHashMap::StringValue(map) => SpecializedHashMap::StringValue(map.clone()),
        }
    }
}

impl HeapObject for SpecializedHashMap {
    fn type_tag(&self) -> TypeTag {
        match self {
            SpecializedHashMap::StringInt(_) => TypeTag::HashMapInt,
            SpecializedHashMap::StringBool(_) => TypeTag::HashMapBool,
            SpecializedHashMap::StringString(_) => TypeTag::HashMapString,
            SpecializedHashMap::StringDouble(_) => TypeTag::SpecializedPair("HashMap_String_double".to_string()),
            SpecializedHashMap::StringValue(_) => TypeTag::HashMapValue,
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// ============================================================================
// HashMap Implementation
// ============================================================================

#[derive(Debug)]
pub struct HashMapData {
    pub data: StdHashMap<String, Value>,
}

pub fn hash_map_new(ctx: &mut VmContext, _capacity: Value) -> Value {
    // Clone the type to avoid holding the borrow across the add_vmref call
    let ty = ctx.lookup_type("HashMap");

    match &ty {
        Type::User(_) => {
            let map_data = HashMapData {
                data: StdHashMap::new(),
            };
            let id = ctx.add_vmref(crate::universe::VmRefData::HashMap(map_data));

            let mut fields = Obj::new();
            fields.set("id", Value::USize(id));
            Value::Instance(Instance {
                ty: auto_val::Type::from(ty),
                fields,
            })
        }
        _ => Value::Error(format!("Type HashMap not found!").into()),
    }
}

pub fn hash_map_insert_str(ctx: &mut VmContext, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();

                        // Safe pattern matching instead of downcasting
                        if let crate::universe::VmRefData::HashMap(map) = &mut *ref_box {
                            if args.len() >= 2 {
                                let key = args[0].to_astr().to_string();
                                let value = args[1].clone();
                                map.data.insert(key, value);
                                return Value::Nil;
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn hash_map_insert_int(ctx: &mut VmContext, instance: &mut Value, args: Vec<Value>) -> Value {
    // Same implementation as insert_str - type is determined at runtime
    hash_map_insert_str(ctx, instance, args)
}

pub fn hash_map_get_str(ctx: &mut VmContext, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let crate::universe::VmRefData::HashMap(map) = &*ref_box {
                            if args.len() >= 1 {
                                let key = args[0].to_astr().to_string();
                                return map.data.get(&key).cloned().unwrap_or(Value::Nil);
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn hash_map_get_int(ctx: &mut VmContext, instance: &mut Value, args: Vec<Value>) -> Value {
    // Same implementation as get_str
    hash_map_get_str(ctx, instance, args)
}

pub fn hash_map_contains(ctx: &mut VmContext, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let crate::universe::VmRefData::HashMap(map) = &*ref_box {
                            if args.len() >= 1 {
                                let key = args[0].to_astr().to_string();
                                return (map.data.contains_key(&key)).into();
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Bool(false)
}

pub fn hash_map_remove(ctx: &mut VmContext, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::HashMap(map) = &mut *ref_box {
                            if args.len() >= 1 {
                                let key = args[0].to_astr().to_string();
                                map.data.remove(&key);
                                return Value::Nil;
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn hash_map_size(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let crate::universe::VmRefData::HashMap(map) = &*ref_box {
                            return Value::Int(map.data.len() as i32);
                        }
                    }
                }
            }
        }
    }
    Value::Int(0)
}

pub fn hash_map_clear(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::HashMap(map) = &mut *ref_box {
                            map.data.clear();
                            return Value::Nil;
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn hash_map_drop(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    ctx.drop_vmref(id);
                    return Value::Nil;
                }
            }
        }
    }
    Value::Nil
}

// Wrapper for static new() method (VmFunction signature - takes single Value)
pub fn hash_map_new_static(ctx: &mut VmContext, _arg: Value) -> Value {
    hash_map_new(ctx, Value::USize(0))
}

// ============================================================================
// HashSet Implementation
// ============================================================================

#[derive(Debug)]
pub struct HashSetData {
    pub data: StdHashMap<String, ()>,
}

// ============================================================================
// VecDeque Implementation (Plan 085)
// ============================================================================

#[derive(Debug)]
pub struct VecDequeData {
    pub data: VecDeque<Value>,
}

// ============================================================================
// BTreeMap Implementation (Plan 085)
// ============================================================================

#[derive(Debug)]
pub struct BTreeMapData {
    pub data: BTreeMap<String, Value>,
}

pub fn hash_set_new(ctx: &mut VmContext, _arg: Value) -> Value {
    // Clone the type to avoid holding the borrow across the add_vmref call
    let ty = ctx.lookup_type("HashSet");

    match &ty {
        Type::User(_) => {
            let set_data = HashSetData {
                data: StdHashMap::new(),
            };
            let id = ctx.add_vmref(crate::universe::VmRefData::HashSet(set_data));
            let mut fields = Obj::new();
            fields.set("id", Value::USize(id));
            Value::Instance(Instance {
                ty: auto_val::Type::from(ty),
                fields,
            })
        }
        _ => Value::Error(format!("Type HashSet not found!").into()),
    }
}

pub fn hash_set_insert(ctx: &mut VmContext, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashSet" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::HashSet(set) = &mut *ref_box {
                            if args.len() >= 1 {
                                let value = args[0].to_astr().to_string();
                                set.data.insert(value, ());
                                return Value::Nil;
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn hash_set_contains(ctx: &mut VmContext, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashSet" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let crate::universe::VmRefData::HashSet(set) = &*ref_box {
                            if args.len() >= 1 {
                                let value = args[0].to_astr().to_string();
                                return (set.data.contains_key(&value)).into();
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Bool(false)
}

pub fn hash_set_remove(ctx: &mut VmContext, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashSet" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::HashSet(set) = &mut *ref_box {
                            if args.len() >= 1 {
                                let value = args[0].to_astr().to_string();
                                set.data.remove(&value);
                                return Value::Nil;
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn hash_set_size(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashSet" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let crate::universe::VmRefData::HashSet(set) = &*ref_box {
                            return Value::Int(set.data.len() as i32);
                        }
                    }
                }
            }
        }
    }
    Value::Int(0)
}

pub fn hash_set_clear(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashSet" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::HashSet(set) = &mut *ref_box {
                            set.data.clear();
                            return Value::Nil;
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn hash_set_drop(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashSet" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    ctx.drop_vmref(id);
                    return Value::Nil;
                }
            }
        }
    }
    Value::Nil
}

// ============================================================================
// VecDeque Implementation (Plan 085)
// ============================================================================

pub fn vec_deque_new(ctx: &mut VmContext, _arg: Value) -> Value {
    let ty = ctx.lookup_type("VecDeque");

    match &ty {
        Type::User(_) => {
            let deque_data = VecDequeData {
                data: VecDeque::new(),
            };
            let id = ctx
                .universe()
                .borrow_mut()
                .add_vmref(crate::universe::VmRefData::VecDeque(deque_data));

            let mut fields = Obj::new();
            fields.set("id", Value::USize(id));
            Value::Instance(Instance {
                ty: auto_val::Type::from(ty),
                fields,
            })
        }
        _ => Value::Error(format!("Type VecDeque not found!").into()),
    }
}

pub fn vec_deque_push_back(ctx: &mut VmContext, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "VecDeque" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::VecDeque(deque) = &mut *ref_box {
                            if args.len() >= 1 {
                                deque.data.push_back(args[0].clone());
                                return Value::Nil;
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn vec_deque_push_front(ctx: &mut VmContext, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "VecDeque" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::VecDeque(deque) = &mut *ref_box {
                            if args.len() >= 1 {
                                deque.data.push_front(args[0].clone());
                                return Value::Nil;
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn vec_deque_pop_back(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "VecDeque" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::VecDeque(deque) = &mut *ref_box {
                            return deque.data.pop_back().unwrap_or(Value::Nil);
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn vec_deque_pop_front(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "VecDeque" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::VecDeque(deque) = &mut *ref_box {
                            return deque.data.pop_front().unwrap_or(Value::Nil);
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn vec_deque_front(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "VecDeque" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let crate::universe::VmRefData::VecDeque(deque) = &*ref_box {
                            return deque.data.front().cloned().unwrap_or(Value::Nil);
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn vec_deque_back(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "VecDeque" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let crate::universe::VmRefData::VecDeque(deque) = &*ref_box {
                            return deque.data.back().cloned().unwrap_or(Value::Nil);
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn vec_deque_size(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "VecDeque" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let crate::universe::VmRefData::VecDeque(deque) = &*ref_box {
                            return Value::Int(deque.data.len() as i32);
                        }
                    }
                }
            }
        }
    }
    Value::Int(0)
}

pub fn vec_deque_is_empty(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "VecDeque" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let crate::universe::VmRefData::VecDeque(deque) = &*ref_box {
                            return deque.data.is_empty().into();
                        }
                    }
                }
            }
        }
    }
    Value::Bool(true)
}

pub fn vec_deque_clear(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "VecDeque" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::VecDeque(deque) = &mut *ref_box {
                            deque.data.clear();
                            return Value::Nil;
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn vec_deque_drop(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "VecDeque" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    ctx.drop_vmref(id);
                    return Value::Nil;
                }
            }
        }
    }
    Value::Nil
}

// ============================================================================
// BTreeMap Implementation (Plan 085)
// ============================================================================

pub fn btree_map_new(ctx: &mut VmContext, _arg: Value) -> Value {
    let ty = ctx.lookup_type("BTreeMap");

    match &ty {
        Type::User(_) => {
            let map_data = BTreeMapData {
                data: BTreeMap::new(),
            };
            let id = ctx
                .universe()
                .borrow_mut()
                .add_vmref(crate::universe::VmRefData::BTreeMap(map_data));

            let mut fields = Obj::new();
            fields.set("id", Value::USize(id));
            Value::Instance(Instance {
                ty: auto_val::Type::from(ty),
                fields,
            })
        }
        _ => Value::Error(format!("Type BTreeMap not found!").into()),
    }
}

pub fn btree_map_insert(ctx: &mut VmContext, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "BTreeMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::BTreeMap(map) = &mut *ref_box {
                            if args.len() >= 2 {
                                let key = args[0].to_astr().to_string();
                                let value = args[1].clone();
                                map.data.insert(key, value);
                                return Value::Nil;
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn btree_map_get(ctx: &mut VmContext, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "BTreeMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let crate::universe::VmRefData::BTreeMap(map) = &*ref_box {
                            if args.len() >= 1 {
                                let key = args[0].to_astr().to_string();
                                return map.data.get(&key).cloned().unwrap_or(Value::Nil);
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn btree_map_contains(ctx: &mut VmContext, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "BTreeMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let crate::universe::VmRefData::BTreeMap(map) = &*ref_box {
                            if args.len() >= 1 {
                                let key = args[0].to_astr().to_string();
                                return (map.data.contains_key(&key)).into();
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Bool(false)
}

pub fn btree_map_remove(ctx: &mut VmContext, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "BTreeMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::BTreeMap(map) = &mut *ref_box {
                            if args.len() >= 1 {
                                let key = args[0].to_astr().to_string();
                                map.data.remove(&key);
                                return Value::Nil;
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn btree_map_size(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "BTreeMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let crate::universe::VmRefData::BTreeMap(map) = &*ref_box {
                            return Value::Int(map.data.len() as i32);
                        }
                    }
                }
            }
        }
    }
    Value::Int(0)
}

pub fn btree_map_is_empty(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "BTreeMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let crate::universe::VmRefData::BTreeMap(map) = &*ref_box {
                            return map.data.is_empty().into();
                        }
                    }
                }
            }
        }
    }
    Value::Bool(true)
}

pub fn btree_map_clear(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "BTreeMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::BTreeMap(map) = &mut *ref_box {
                            map.data.clear();
                            return Value::Nil;
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn btree_map_first_key(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "BTreeMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let crate::universe::VmRefData::BTreeMap(map) = &*ref_box {
                            if let Some(key) = map.data.keys().next() {
                                return Value::Str(key.clone().into());
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn btree_map_last_key(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "BTreeMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = ctx.universe(); let uni_ref = uni.borrow();
                    let b = uni_ref.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let crate::universe::VmRefData::BTreeMap(map) = &*ref_box {
                            if let Some(key) = map.data.keys().next_back() {
                                return Value::Str(key.clone().into());
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn btree_map_drop(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "BTreeMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    ctx.drop_vmref(id);
                    return Value::Nil;
                }
            }
        }
    }
    Value::Nil
}

// ============================================================================
// AutoVM HeapObject Implementation (Plan 086)
// ============================================================================

/// AutoVM HashMap - stores string keys to i32 values
/// Simplified version for AutoVM bytecode execution
#[derive(Debug)]
pub struct AutoVMHashMap {
    pub data: StdHashMap<String, i32>,
}

impl AutoVMHashMap {
    pub fn new() -> Self {
        Self {
            data: StdHashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: StdHashMap::with_capacity(capacity),
        }
    }
}

impl HeapObject for AutoVMHashMap {
    fn type_tag(&self) -> TypeTag { TypeTag::HashMapInt }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// ============================================================================
// Plan 087 Phase 4: SpecializedHashMap Tests
// ============================================================================

#[cfg(test)]
mod specialized_hashmap_tests {
    use super::*;
    use auto_val::AutoStr;

    #[test]
    fn test_specialized_hash_map_string_int_creation() {
        let map = SpecializedHashMap::new("int");
        match map {
            SpecializedHashMap::StringInt(_) => {}
            _ => panic!("Expected StringInt variant"),
        }
    }

    #[test]
    fn test_specialized_hash_map_string_bool_creation() {
        let map = SpecializedHashMap::new("bool");
        match map {
            SpecializedHashMap::StringBool(_) => {}
            _ => panic!("Expected StringBool variant"),
        }
    }

    #[test]
    fn test_specialized_hash_map_string_string_creation() {
        let map = SpecializedHashMap::new("string");
        match map {
            SpecializedHashMap::StringString(_) => {}
            _ => panic!("Expected StringString variant"),
        }
    }

    #[test]
    fn test_specialized_hash_map_string_double_creation() {
        let map = SpecializedHashMap::new("double");
        match map {
            SpecializedHashMap::StringDouble(_) => {}
            _ => panic!("Expected StringDouble variant"),
        }
    }

    #[test]
    fn test_specialized_hash_map_string_value_creation() {
        let map = SpecializedHashMap::new("unknown");
        match map {
            SpecializedHashMap::StringValue(_) => {}
            _ => panic!("Expected StringValue variant"),
        }
    }

    #[test]
    fn test_specialized_hash_map_insert_int() {
        let mut map = SpecializedHashMap::new("int");
        map.insert("key1".to_string(), Value::Int(42)).unwrap();
        map.insert("key2".to_string(), Value::Int(100)).unwrap();

        assert_eq!(map.len(), 2);
        assert_eq!(map.get("key1"), Some(Value::Int(42)));
        assert_eq!(map.get("key2"), Some(Value::Int(100)));
        assert_eq!(map.get("key3"), None);
    }

    #[test]
    fn test_specialized_hash_map_insert_bool() {
        let mut map = SpecializedHashMap::new("bool");
        map.insert("flag1".to_string(), Value::Bool(true)).unwrap();
        map.insert("flag2".to_string(), Value::Bool(false)).unwrap();

        assert_eq!(map.len(), 2);
        assert_eq!(map.get("flag1"), Some(Value::Bool(true)));
        assert_eq!(map.get("flag2"), Some(Value::Bool(false)));
    }

    #[test]
    fn test_specialized_hash_map_insert_string() {
        let mut map = SpecializedHashMap::new("string");
        map.insert("name".to_string(), Value::Str(AutoStr::from("Alice"))).unwrap();
        map.insert("city".to_string(), Value::Str(AutoStr::from("Bob"))).unwrap();

        assert_eq!(map.len(), 2);
        assert_eq!(map.get("name"), Some(Value::Str(AutoStr::from("Alice"))));
        assert_eq!(map.get("city"), Some(Value::Str(AutoStr::from("Bob"))));
    }

    #[test]
    fn test_specialized_hash_map_insert_double() {
        let mut map = SpecializedHashMap::new("double");
        map.insert("pi".to_string(), Value::Double(3.14159)).unwrap();
        map.insert("e".to_string(), Value::Double(2.71828)).unwrap();

        assert_eq!(map.len(), 2);
        assert_eq!(map.get("pi"), Some(Value::Double(3.14159)));
        assert_eq!(map.get("e"), Some(Value::Double(2.71828)));
    }

    #[test]
    fn test_specialized_hash_map_insert_value() {
        let mut map = SpecializedHashMap::new("unknown");
        map.insert("int_val".to_string(), Value::Int(42)).unwrap();
        map.insert("bool_val".to_string(), Value::Bool(true)).unwrap();
        map.insert("str_val".to_string(), Value::Str(AutoStr::from("hello"))).unwrap();

        assert_eq!(map.len(), 3);
        assert_eq!(map.get("int_val"), Some(Value::Int(42)));
        assert_eq!(map.get("bool_val"), Some(Value::Bool(true)));
        assert_eq!(map.get("str_val"), Some(Value::Str(AutoStr::from("hello"))));
    }

    #[test]
    fn test_specialized_hash_map_type_mismatch() {
        let mut map = SpecializedHashMap::new("int");
        let result = map.insert("key".to_string(), Value::Str(AutoStr::from("wrong")));
        assert!(result.is_err());
    }

    #[test]
    fn test_specialized_hash_map_contains_key() {
        let mut map = SpecializedHashMap::new("int");
        map.insert("key1".to_string(), Value::Int(42)).unwrap();

        assert!(map.contains_key("key1"));
        assert!(!map.contains_key("key2"));
    }

    #[test]
    fn test_specialized_hash_map_remove() {
        let mut map = SpecializedHashMap::new("int");
        map.insert("key1".to_string(), Value::Int(42)).unwrap();
        map.insert("key2".to_string(), Value::Int(100)).unwrap();

        assert_eq!(map.len(), 2);

        let removed = map.remove("key1");
        assert_eq!(removed, Some(Value::Int(42)));
        assert_eq!(map.len(), 1);
        assert!(!map.contains_key("key1"));

        let removed = map.remove("nonexistent");
        assert_eq!(removed, None);
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_specialized_hash_map_is_empty() {
        let mut map = SpecializedHashMap::new("int");
        assert!(map.is_empty());

        map.insert("key".to_string(), Value::Int(42)).unwrap();
        assert!(!map.is_empty());

        map.remove("key");
        assert!(map.is_empty());
    }

    #[test]
    fn test_specialized_hash_map_clear() {
        let mut map = SpecializedHashMap::new("bool");
        map.insert("flag1".to_string(), Value::Bool(true)).unwrap();
        map.insert("flag2".to_string(), Value::Bool(false)).unwrap();

        assert_eq!(map.len(), 2);

        map.clear();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_specialized_hash_map_clone() {
        let mut map = SpecializedHashMap::new("int");
        map.insert("key1".to_string(), Value::Int(42)).unwrap();
        map.insert("key2".to_string(), Value::Int(100)).unwrap();

        let cloned = map.clone();
        assert_eq!(cloned.len(), 2);
        assert_eq!(cloned.get("key1"), Some(Value::Int(42)));
        assert_eq!(cloned.get("key2"), Some(Value::Int(100)));

        // Verify independence
        map.insert("key3".to_string(), Value::Int(200)).unwrap();
        assert_eq!(map.len(), 3);
        assert_eq!(cloned.len(), 2);
    }

    #[test]
    fn test_specialized_hash_map_overwrite() {
        let mut map = SpecializedHashMap::new("int");
        map.insert("key".to_string(), Value::Int(42)).unwrap();
        assert_eq!(map.get("key"), Some(Value::Int(42)));

        map.insert("key".to_string(), Value::Int(100)).unwrap();
        assert_eq!(map.get("key"), Some(Value::Int(100)));
        assert_eq!(map.len(), 1); // Still only 1 entry
    }

    #[test]
    fn test_specialized_hash_map_memory_efficiency() {
        // Verify that specialized variants use primitive types
        let map_int = SpecializedHashMap::StringInt(StdHashMap::new());
        match map_int {
            SpecializedHashMap::StringInt(_) => {
                // Values are stored as i32 (4 bytes) instead of Value (24+ bytes)
                assert_eq!(std::mem::size_of::<i32>(), 4);
                assert!(std::mem::size_of::<Value>() > std::mem::size_of::<i32>());
            }
            _ => panic!("Expected StringInt variant"),
        }

        let map_bool = SpecializedHashMap::StringBool(StdHashMap::new());
        match map_bool {
            SpecializedHashMap::StringBool(_) => {
                // Values are stored as bool (1 byte) instead of Value (24+ bytes)
                assert_eq!(std::mem::size_of::<bool>(), 1);
                assert!(std::mem::size_of::<Value>() > std::mem::size_of::<bool>());
            }
            _ => panic!("Expected StringBool variant"),
        }
    }
}
