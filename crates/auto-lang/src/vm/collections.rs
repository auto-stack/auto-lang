use auto_val::{Instance, Obj, Type, Value};
use std::collections::{BTreeMap, HashMap as StdHashMap, VecDeque};

use crate::{ast, eval::Evaler};

// ============================================================================
// HashMap Implementation
// ============================================================================

#[derive(Debug)]
pub struct HashMapData {
    pub data: StdHashMap<String, Value>,
}

pub fn hash_map_new(_evaler: &mut Evaler, _capacity: Value) -> Value {
    // Clone the type to avoid holding the borrow across the add_vmref call
    let ty = _evaler.lookup_type("HashMap");

    match &ty {
        ast::Type::User(_) => {
            let map_data = HashMapData {
                data: StdHashMap::new(),
            };
            let id = _evaler.universe().borrow_mut().add_vmref(crate::universe::VmRefData::HashMap(map_data));

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

pub fn hash_map_insert_str(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn hash_map_insert_int(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    // Same implementation as insert_str - type is determined at runtime
    hash_map_insert_str(_evaler, instance, args)
}

pub fn hash_map_get_str(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn hash_map_get_int(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    // Same implementation as get_str
    hash_map_get_str(_evaler, instance, args)
}

pub fn hash_map_contains(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn hash_map_remove(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn hash_map_size(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn hash_map_clear(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn hash_map_drop(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    _evaler.universe().borrow_mut().drop_vmref(id);
                    return Value::Nil;
                }
            }
        }
    }
    Value::Nil
}

// Wrapper for static new() method (VmFunction signature - takes single Value)
pub fn hash_map_new_static(_evaler: &mut Evaler, _arg: Value) -> Value {
    hash_map_new(_evaler, Value::USize(0))
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

pub fn hash_set_new(_evaler: &mut Evaler, _arg: Value) -> Value {
    // Clone the type to avoid holding the borrow across the add_vmref call
    let ty = _evaler.lookup_type("HashSet");

    match &ty {
        ast::Type::User(_) => {
            let set_data = HashSetData {
                data: StdHashMap::new(),
            };
            let id = _evaler.universe().borrow_mut().add_vmref(crate::universe::VmRefData::HashSet(set_data));
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

pub fn hash_set_insert(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashSet" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn hash_set_contains(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashSet" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn hash_set_remove(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashSet" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn hash_set_size(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashSet" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn hash_set_clear(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashSet" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn hash_set_drop(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashSet" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    _evaler.universe().borrow_mut().drop_vmref(id);
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

pub fn vec_deque_new(_evaler: &mut Evaler, _arg: Value) -> Value {
    let ty = _evaler.lookup_type("VecDeque");

    match &ty {
        ast::Type::User(_) => {
            let deque_data = VecDequeData {
                data: VecDeque::new(),
            };
            let id = _evaler
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

pub fn vec_deque_push_back(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "VecDeque" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn vec_deque_push_front(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "VecDeque" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn vec_deque_pop_back(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "VecDeque" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn vec_deque_pop_front(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "VecDeque" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn vec_deque_front(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "VecDeque" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn vec_deque_back(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "VecDeque" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn vec_deque_size(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "VecDeque" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn vec_deque_is_empty(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "VecDeque" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn vec_deque_clear(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "VecDeque" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn vec_deque_drop(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "VecDeque" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    _evaler.universe().borrow_mut().drop_vmref(id);
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

pub fn btree_map_new(_evaler: &mut Evaler, _arg: Value) -> Value {
    let ty = _evaler.lookup_type("BTreeMap");

    match &ty {
        ast::Type::User(_) => {
            let map_data = BTreeMapData {
                data: BTreeMap::new(),
            };
            let id = _evaler
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

pub fn btree_map_insert(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "BTreeMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn btree_map_get(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "BTreeMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn btree_map_contains(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "BTreeMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn btree_map_remove(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "BTreeMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn btree_map_size(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "BTreeMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn btree_map_is_empty(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "BTreeMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn btree_map_clear(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "BTreeMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn btree_map_first_key(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "BTreeMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn btree_map_last_key(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "BTreeMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
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

pub fn btree_map_drop(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "BTreeMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    _evaler.universe().borrow_mut().drop_vmref(id);
                    return Value::Nil;
                }
            }
        }
    }
    Value::Nil
}
