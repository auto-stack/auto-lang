use auto_val::{Instance, Obj, Shared, Type, Value};
use crate::{ast, Universe};

// ============================================================================
// StringBuilder Implementation
// ============================================================================

#[derive(Debug)]
pub struct StringBuilderData {
    pub buffer: String,
}

pub fn string_builder_new(uni: Shared<Universe>, capacity: Value) -> Value {
    let ty = uni.borrow().lookup_type("StringBuilder");
    match &ty {
        ast::Type::User(_) => {
            let _cap = if let Value::Int(c) = capacity {
                c as usize
            } else {
                1024
            };

            let builder_data = StringBuilderData {
                buffer: String::with_capacity(_cap),
            };
            let id = uni.borrow_mut().add_vmref(crate::universe::VmRefData::StringBuilder(builder_data));
            let mut fields = Obj::new();
            fields.set("id", Value::USize(id));
            Value::Instance(Instance {
                ty: auto_val::Type::from(ty),
                fields,
            })
        }
        _ => Value::Error(format!("Type StringBuilder not found!").into()),
    }
}

pub fn string_builder_append(uni: Shared<Universe>, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "StringBuilder" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
                    let b = uni.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::StringBuilder(builder) = &mut *ref_box {
                            if args.len() >= 1 {
                                let s = args[0].to_astr();
                                builder.buffer.push_str(s.as_str());
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

pub fn string_builder_append_char(uni: Shared<Universe>, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "StringBuilder" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
                    let b = uni.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::StringBuilder(builder) = &mut *ref_box {
                            if args.len() >= 1 {
                                if let Value::Char(c) = args[0] {
                                    builder.buffer.push(c);
                                    return Value::Nil;
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

pub fn string_builder_append_int(uni: Shared<Universe>, instance: &mut Value, args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "StringBuilder" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
                    let b = uni.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::StringBuilder(builder) = &mut *ref_box {
                            if args.len() >= 1 {
                                let s = args[0].to_astr();
                                builder.buffer.push_str(s.as_str());
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

pub fn string_builder_build(uni: Shared<Universe>, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "StringBuilder" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
                    let b = uni.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let crate::universe::VmRefData::StringBuilder(builder) = &*ref_box {
                            return Value::Str(builder.buffer.clone().into());
                        }
                    }
                }
            }
        }
    }
    Value::empty_str()
}

pub fn string_builder_clear(uni: Shared<Universe>, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "StringBuilder" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
                    let b = uni.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::StringBuilder(builder) = &mut *ref_box {
                            builder.buffer.clear();
                            return Value::Nil;
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn string_builder_len(uni: Shared<Universe>, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "StringBuilder" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = uni.borrow();
                    let b = uni.get_vmref_ref(id);
                    if let Some(b) = b {
                        let ref_box = b.borrow();
                        if let crate::universe::VmRefData::StringBuilder(builder) = &*ref_box {
                            return Value::Int(builder.buffer.len() as i32);
                        }
                    }
                }
            }
        }
    }
    Value::Int(0)
}

pub fn string_builder_drop(uni: Shared<Universe>, instance: &mut Value, _args: Vec<Value>) -> Value {
    if let Value::Instance(inst) = instance {
        if let Type::User(decl) = &inst.ty {
            if decl == "StringBuilder" {
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
pub fn string_builder_new_static(uni: Shared<Universe>, arg: Value) -> Value {
    string_builder_new(uni, arg)
}

// Wrapper for static new_with_default() method (VmFunction signature - takes single Value)
pub fn string_builder_new_with_default_static(uni: Shared<Universe>, _arg: Value) -> Value {
    string_builder_new(uni, Value::Int(1024))
}
