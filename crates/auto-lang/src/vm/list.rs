//! Dynamic list (Vec-like) operations for AutoLang
//!
//! Provides VM methods for the List<T> type, which is a growable, heap-allocated
//! dynamic array similar to Rust's Vec<T> or Python's list.

use crate::{ast, Universe};
use crate::universe::{ListData, VmRefData};
use auto_val::{Instance, Obj, Shared, Type, Value};

/// Create a new empty List
/// Syntax: List.new()
pub fn list_new_static(uni: Shared<Universe>, _arg: Value) -> Value {
    list_new(uni, Value::USize(0))
}

/// Create a new List with specified capacity
pub fn list_new(uni: Shared<Universe>, _capacity: Value) -> Value {
    // Clone the type to avoid holding the borrow across the add_vmref call
    let ty = {
        let uni_borrow = uni.borrow();
        uni_borrow.lookup_type("List")
    };

    match &ty {
        ast::Type::User(_) => {
            let list_data = ListData {
                elems: Vec::new(),
            };
            let id = uni.borrow_mut().add_vmref(crate::universe::VmRefData::List(list_data));

            let mut fields = Obj::new();
            fields.set("id", Value::USize(id));
            Value::Instance(Instance {
                ty: auto_val::Type::from(ty),
                fields,
            })
        }
        _ => Value::Error(format!("Type List not found!").into()),
    }
}

/// Push element to List
/// Syntax: list.push(elem)
pub fn list_push(uni: Shared<Universe>, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(ref_name) = &inst.ty {
            if ref_name == "List" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
                    let b = uni.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let VmRefData::List(list) = &mut *ref_box {
                            if let Some(elem) = args.first() {
                                list.push(elem.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

/// Pop element from List
pub fn list_pop(uni: Shared<Universe>, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(ref_name) = &inst.ty {
            if ref_name == "List" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
                    let b = uni.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let VmRefData::List(list) = &mut *ref_box {
                            return list.pop().unwrap_or(Value::Nil);
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

/// Get length of List
pub fn list_len(uni: Shared<Universe>, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(ref_name) = &inst.ty {
            if ref_name == "List" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
                    let b = uni.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let VmRefData::List(list) = &*ref_box {
                            return Value::Int(list.len() as i32);
                        }
                    }
                }
            }
        }
    }
    Value::Int(0)
}

/// Check if List is empty
pub fn list_is_empty(uni: Shared<Universe>, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(ref_name) = &inst.ty {
            if ref_name == "List" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
                    let b = uni.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let VmRefData::List(list) = &*ref_box {
                            return Value::Int(if list.is_empty() { 1 } else { 0 });
                        }
                    }
                }
            }
        }
    }
    Value::Int(1)
}

/// Clear all elements
pub fn list_clear(uni: Shared<Universe>, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(ref_name) = &inst.ty {
            if ref_name == "List" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
                    let b = uni.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let VmRefData::List(list) = &mut *ref_box {
                            list.clear();
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

/// Reserve capacity
pub fn list_reserve(uni: Shared<Universe>, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(ref_name) = &inst.ty {
            if ref_name == "List" {
                if let Some(cap) = args.first() {
                    if let Value::Int(cap_value) = cap {
                        let id = inst.fields.get("id");
                        if let Some(Value::USize(id)) = id {
                            let uni = uni.borrow();
                            let b = uni.get_vmref_ref(id);
                            if let Some(b) = b {
                                let mut ref_box = b.borrow_mut();
                                if let VmRefData::List(list) = &mut *ref_box {
                                    list.reserve(*cap_value as usize);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

/// Get element at index
pub fn list_get(uni: Shared<Universe>, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(ref_name) = &inst.ty {
            if ref_name == "List" {
                if let Some(idx) = args.first() {
                    if let Value::Int(idx_value) = idx {
                        let id = inst.fields.get("id");
                        if let Some(Value::USize(id)) = id {
                            let uni = uni.borrow();
                            let b = uni.get_vmref_ref(id);
                            if let Some(b) = b {
                                let ref_box = b.borrow();
                                if let VmRefData::List(list) = &*ref_box {
                                    return list.get(*idx_value as usize)
                                        .cloned()
                                        .unwrap_or(Value::Nil);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

/// Set element at index
pub fn list_set(uni: Shared<Universe>, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(ref_name) = &inst.ty {
            if ref_name == "List" {
                if args.len() >= 2 {
                    if let Value::Int(idx_value) = &args[0] {
                        let elem = &args[1];
                        let id = inst.fields.get("id");
                        if let Some(Value::USize(id)) = id {
                            let uni = uni.borrow();
                            let b = uni.get_vmref_ref(id);
                            if let Some(b) = b {
                                let mut ref_box = b.borrow_mut();
                                if let VmRefData::List(list) = &mut *ref_box {
                                    let success = list.set(*idx_value as usize, elem.clone());
                                    return Value::Int(if success { 1 } else { 0 });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Int(0)
}

/// Insert element at index
pub fn list_insert(uni: Shared<Universe>, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(ref_name) = &inst.ty {
            if ref_name == "List" {
                if args.len() >= 2 {
                    if let Value::Int(idx_value) = &args[0] {
                        let elem = &args[1];
                        let id = inst.fields.get("id");
                        if let Some(Value::USize(id)) = id {
                            let uni = uni.borrow();
                            let b = uni.get_vmref_ref(id);
                            if let Some(b) = b {
                                let mut ref_box = b.borrow_mut();
                                if let VmRefData::List(list) = &mut *ref_box {
                                    list.insert(*idx_value as usize, elem.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

/// Remove element at index
pub fn list_remove(uni: Shared<Universe>, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(ref_name) = &inst.ty {
            if ref_name == "List" {
                if let Some(idx) = args.first() {
                    if let Value::Int(idx_value) = idx {
                        let id = inst.fields.get("id");
                        if let Some(Value::USize(id)) = id {
                            let uni = uni.borrow();
                            let b = uni.get_vmref_ref(id);
                            if let Some(b) = b {
                                let mut ref_box = b.borrow_mut();
                                if let VmRefData::List(list) = &mut *ref_box {
                                    return list.remove(*idx_value as usize).unwrap_or(Value::Nil);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

/// Drop the List and free its resources
pub fn list_drop(uni: Shared<Universe>, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(ref_name) = &inst.ty {
            if ref_name == "List" {
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
