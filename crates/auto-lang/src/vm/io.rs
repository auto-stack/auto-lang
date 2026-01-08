use auto_val::{Instance, Obj, Shared, Type, Value};
use std::{fs::File, io::Read};

use crate::{ast, Universe};

pub fn open(uni: Shared<Universe>, path: Value) -> Value {
    match path {
        Value::Str(p) => {
            let f = File::open(p.as_str());
            match f {
                Ok(file) => {
                    let ty = uni.borrow().lookup_type("File");
                    match &ty {
                        ast::Type::User(_) => {
                            let b = Box::new(file);
                            let id = uni.borrow_mut().add_vmref(b);
                            let mut fields = Obj::new();
                            fields.set("id", Value::USize(id));
                            Value::Instance(Instance {
                                ty: auto_val::Type::from(ty),
                                fields,
                            })
                        }
                        _ => Value::Error(format!("Type File not found!").into()),
                    }
                }
                Err(e) => Value::Error(format!("File {} not found: {}", p, e).into()),
            }
        }
        _ => Value::Nil,
    }
}

pub fn read_text(uni: Shared<Universe>, file: &mut Value) -> Value {
    if let Value::Instance(inst) = file {
        if let Type::User(decl) = &inst.ty {
            if decl == "File" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let mut uni = uni.borrow_mut();
                    let b = uni.get_vmref(id);
                    if let Some(b) = b {
                        if let Some(mut f) = b.downcast_ref::<File>() {
                            let mut s = String::new();
                            if let Ok(_) = f.read_to_string(&mut s) {
                                return Value::Str(s.into());
                            }
                        }
                    }
                }
            }
        }
    }
    Value::empty_str()
}

pub fn close(uni: Shared<Universe>, file: &mut Value) -> Value {
    if let Value::Instance(inst) = file {
        if let Type::User(decl) = &inst.ty {
            if decl == "File" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let mut uni = uni.borrow_mut();
                    uni.drop_vmref(id);
                };
            }
        }
    }
    Value::Nil
}

/// Wrapper for read_text to match VmMethod signature
pub fn read_text_method(uni: Shared<Universe>, instance: &mut Value, _args: Vec<Value>) -> Value {
    read_text(uni, instance)
}

/// Wrapper for close to match VmMethod signature
pub fn close_method(uni: Shared<Universe>, instance: &mut Value, _args: Vec<Value>) -> Value {
    close(uni, instance)
}
