use auto_val::{Instance, Obj, Shared, Type, Value};
use std::collections::HashMap as StdHashMap;

use crate::{ast, Universe};

// ============================================================================
// HashMap Implementation
// ============================================================================

#[derive(Debug)]
pub struct HashMapData {
    pub data: StdHashMap<String, Value>,
}

pub fn hash_map_new(uni: Shared<Universe>, _capacity: Value) -> Value {
    // Clone the type to avoid holding the borrow across the add_vmref call
    let ty = {
        let uni_borrow = uni.borrow();
        uni_borrow.lookup_type("HashMap")
    };

    match &ty {
        ast::Type::User(_) => {
            let map_data = HashMapData {
                data: StdHashMap::new(),
            };
            let id = uni.borrow_mut().add_vmref(crate::universe::VmRefData::HashMap(map_data));

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

pub fn hash_map_insert_str(uni: Shared<Universe>, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
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

pub fn hash_map_insert_int(uni: Shared<Universe>, instance: &mut Value, args: Vec<Value>) -> Value {
    // Same implementation as insert_str - type is determined at runtime
    hash_map_insert_str(uni, instance, args)
}

pub fn hash_map_get_str(uni: Shared<Universe>, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
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

pub fn hash_map_get_int(uni: Shared<Universe>, instance: &mut Value, args: Vec<Value>) -> Value {
    // Same implementation as get_str
    hash_map_get_str(uni, instance, args)
}

pub fn hash_map_contains(uni: Shared<Universe>, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
                    let b = uni.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let crate::universe::VmRefData::HashMap(map) = &*ref_box {
                            if args.len() >= 1 {
                                let key = args[0].to_astr().to_string();
                                return Value::Int(if map.data.contains_key(&key) { 1 } else { 0 });
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Int(0)
}

pub fn hash_map_remove(uni: Shared<Universe>, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
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

pub fn hash_map_size(uni: Shared<Universe>, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
                    let b = uni.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let crate::universe::VmRefData::HashMap(map) = &*ref_box {
                            return Value::USize(map.data.len());
                        }
                    }
                }
            }
        }
    }
    Value::USize(0)
}

pub fn hash_map_clear(uni: Shared<Universe>, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
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

pub fn hash_map_drop(uni: Shared<Universe>, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashMap" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let mut uni = uni.borrow_mut();
                    uni.drop_vmref(id);
                    return Value::Nil;
                }
            }
        }
    }
    Value::Nil
}

// Wrapper for static new() method (VmFunction signature - takes single Value)
pub fn hash_map_new_static(uni: Shared<Universe>, _arg: Value) -> Value {
    hash_map_new(uni, Value::USize(0))
}

// ============================================================================
// HashSet Implementation
// ============================================================================

#[derive(Debug)]
pub struct HashSetData {
    pub data: StdHashMap<String, ()>,
}

pub fn hash_set_new(uni: Shared<Universe>, _arg: Value) -> Value {
    // Clone the type to avoid holding the borrow across the add_vmref call
    let ty = {
        let uni_borrow = uni.borrow();
        uni_borrow.lookup_type("HashSet")
    };

    match &ty {
        ast::Type::User(_) => {
            let set_data = HashSetData {
                data: StdHashMap::new(),
            };
            let id = uni.borrow_mut().add_vmref(crate::universe::VmRefData::HashSet(set_data));
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

pub fn hash_set_insert(uni: Shared<Universe>, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashSet" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
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

pub fn hash_set_contains(uni: Shared<Universe>, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashSet" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
                    let b = uni.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let crate::universe::VmRefData::HashSet(set) = &*ref_box {
                            if args.len() >= 1 {
                                let value = args[0].to_astr().to_string();
                                return Value::Int(if set.data.contains_key(&value) { 1 } else { 0 });
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Int(0)
}

pub fn hash_set_remove(uni: Shared<Universe>, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashSet" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
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

pub fn hash_set_size(uni: Shared<Universe>, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashSet" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
                    let b = uni.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let crate::universe::VmRefData::HashSet(set) = &*ref_box {
                            return Value::USize(set.data.len());
                        }
                    }
                }
            }
        }
    }
    Value::USize(0)
}

pub fn hash_set_clear(uni: Shared<Universe>, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashSet" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
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

pub fn hash_set_drop(uni: Shared<Universe>, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "HashSet" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let mut uni = uni.borrow_mut();
                    uni.drop_vmref(id);
                    return Value::Nil;
                }
            }
        }
    }
    Value::Nil
}
