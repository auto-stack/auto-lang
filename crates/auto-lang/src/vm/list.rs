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

/// Create a new List with optional initial elements
/// Syntax: List.new() or List.new(1, 2, 3)
///
/// This function supports varargs for initialization:
/// - List.new() creates an empty list
/// - List.new(1, 2, 3) creates a list with elements [1, 2, 3]
pub fn list_new(uni: Shared<Universe>, initial: Value) -> Value {
    // Clone the type to avoid holding the borrow across the add_vmref call
    let ty = {
        let uni_borrow = uni.borrow();
        uni_borrow.lookup_type("List")
    };

    match &ty {
        ast::Type::User(_) => {
            // Parse initial elements from the argument
            let mut elems = Vec::with_capacity(4);  // Pre-allocate initial capacity of 4

            // Check if initial is an array (multiple arguments passed)
            if let Value::Array(array) = &initial {
                for v in &array.values {
                    elems.push(v.clone());
                }
            }
            // If initial is Nil, create empty list
            // Otherwise, single element initialization

            let list_data = ListData { elems };
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

/// Get the actual capacity of the list's underlying Vec
/// Returns the allocated capacity (may be greater than len())
pub fn list_capacity(uni: Shared<Universe>, instance: &mut Value, _args: Vec<Value>) -> Value {
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
                            return Value::Int(list.elems.capacity() as i32);
                        }
                    }
                }
            }
        }
    }
    Value::Int(0)
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

/// Create an iterator for the List
/// Syntax: list.iter()
/// Returns a ListIter instance
pub fn list_iter(_uni: Shared<Universe>, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(ref_name) = &inst.ty {
            if ref_name == "List" {
                // Get the list ID
                let id: usize = match inst.fields.get("id") {
                    Some(Value::USize(id_val)) => id_val,
                    None => 0,
                    _ => 0,
                };

                // Create ListIter instance with list_id and initial index 0
                let mut fields = Obj::new();
                fields.set("list_id", Value::USize(id));
                fields.set("index", Value::USize(0));

                Value::Instance(Instance {
                    ty: auto_val::Type::User("ListIter".into()),
                    fields,
                })
            } else {
                Value::Nil
            }
        } else {
            Value::Nil
        }
    } else {
        Value::Nil
    }
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

// ============================================================================
// ListIter Methods (Plan 051 Phase 2)
// ============================================================================

/// Get the next element from the iterator
/// Syntax: iter.next()
/// Returns the next element or nil when iteration is complete
pub fn list_iter_next(uni: Shared<Universe>, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(ref_name) = &inst.ty {
            if ref_name == "ListIter" {
                let list_id = inst.fields.get("list_id");
                let index = inst.fields.get("index");

                let (list_id, idx): (usize, usize) = match (list_id, index) {
                    (Some(Value::USize(lid)), Some(Value::USize(iid))) => (lid, iid),
                    _ => (0, 0),
                };

                let uni = uni.borrow();
                let b = uni.get_vmref_ref(list_id);
                if let Some(b) = b {
                    let mut ref_box = b.borrow_mut();
                    if let VmRefData::List(list) = &mut *ref_box {
                        if idx < list.elems.len() {
                            // Get the element at current index
                            let elem = list.elems.get(idx).cloned().unwrap_or(Value::Nil);

                            // Increment index in the iterator
                            drop(ref_box); // Drop borrow before modifying instance
                            drop(uni);
                            inst.fields.set("index", Value::USize(idx + 1));

                            return elem;
                        } else {
                            // End of iteration
                            return Value::Nil;
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}
