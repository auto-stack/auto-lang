use auto_val::{Instance, Obj, Shared, Type, Value};
use std::{
    fs::File,
    io::{BufRead, Read},
};

use crate::{ast, eval::Evaler};

pub fn open(_evaler: &mut Evaler, path: Value) -> Value {
    match path {
        Value::Str(p) => {
            let f = File::open(p.as_str());
            match f {
                Ok(file) => {
                    let ty = _evaler.lookup_type("File");
                    match &ty {
                        ast::Type::User(_) => {
                            let reader = std::io::BufReader::new(file);
                            let id = _evaler.universe().borrow_mut().add_vmref(crate::universe::VmRefData::File(reader));
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
        Value::OwnedStr(p) => {
            let f = File::open(p.as_str());
            match f {
                Ok(file) => {
                    let ty = _evaler.lookup_type("File");
                    match &ty {
                        ast::Type::User(_) => {
                            let reader = std::io::BufReader::new(file);
                            let id = _evaler.universe().borrow_mut().add_vmref(crate::universe::VmRefData::File(reader));
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
                Err(e) => Value::Error(format!("File {} not found: {}", p.as_str(), e).into()),
            }
        }
        _ => Value::Nil,
    }
}

pub fn read_text(_evaler: &mut Evaler, file: &mut Value) -> Value {
    if let Value::Instance(inst) = file {
        if let Type::User(decl) = &inst.ty {
            if decl == "File" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::File(f) = &mut *ref_box {
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

pub fn read_line(_evaler: &mut Evaler, file: &mut Value) -> Value {
    if let Value::Instance(inst) = file {
        if let Type::User(decl) = &inst.ty {
            if decl == "File" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::File(f) = &mut *ref_box {
                            // f is now &mut BufReader<File>, which implements BufRead
                            let mut line = String::new();
                            return match f.read_line(&mut line) {
                                Ok(0) => Value::empty_str(), // EOF
                                Ok(_) => {
                                    // Remove trailing newline if present
                                    if line.ends_with('\n') {
                                        line.pop();
                                        if line.ends_with('\r') {
                                            line.pop();
                                        }
                                    }
                                    Value::Str(line.into())
                                }
                                Err(_) => Value::Error("Failed to read line".into()),
                            };
                        }
                    }
                }
            }
        }
    }
    Value::empty_str()
}

pub fn close(_evaler: &mut Evaler, file: &mut Value) -> Value {
    if let Value::Instance(inst) = file {
        if let Type::User(decl) = &inst.ty {
            if decl == "File" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    _evaler.universe().borrow_mut().drop_vmref(id);
                };
            }
        }
    }
    Value::Nil
}

/// Wrapper for read_text to match VmMethod signature
pub fn read_text_method(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    read_text(_evaler, instance)
}

/// Wrapper for close to match VmMethod signature
pub fn close_method(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    close(_evaler, instance)
}

pub fn read_line_method(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    read_line(_evaler, instance)
}

pub fn read_char(_evaler: &mut Evaler, file: &mut Value) -> Value {
    if let Value::Instance(inst) = file {
        if let Type::User(decl) = &inst.ty {
            if decl == "File" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::File(f) = &mut *ref_box {
                            let mut buf = [0u8; 1];
                            return match f.read(&mut buf) {
                                Ok(0) => Value::Int(-1), // EOF
                                Ok(_) => Value::Int(buf[0] as i32),
                                Err(_) => Value::Int(-1),
                            };
                        }
                    }
                }
            }
        }
    }
    Value::Int(-1)
}

pub fn read_char_method(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    read_char(_evaler, instance)
}

pub fn read_buf(_evaler: &mut Evaler, _file: &mut Value, _buf: &mut Value, _size: i64) -> Value {
    // VM does not support read_buf with mutable string buffer yet for immutable str
    Value::Int(0)
}

pub fn read_buf_method(_evaler: &mut Evaler, _instance: &mut Value, _args: Vec<Value>) -> Value {
    // Stub implementation
    Value::Int(0)
}

pub fn write_line(_evaler: &mut Evaler, file: &mut Value, line: &str) -> Value {
    if let Value::Instance(inst) = file {
        if let Type::User(decl) = &inst.ty {
            if decl == "File" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::File(f) = &mut *ref_box {
                            use std::io::Write;
                            if let Err(e) = writeln!(f.get_mut(), "{}", line) {
                                return Value::Error(format!("Write error: {}", e).into());
                            }
                            return Value::Nil;
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn write_line_method(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    let line = if let Some(val) = args.get(0) {
        match val {
            Value::Str(s) => s.as_str(),
            Value::OwnedStr(s) => s.as_str(),
            _ => return Value::Error("Argument must be a string".into()),
        }
    } else {
        return Value::Error("Missing argument".into());
    };
    write_line(_evaler, instance, line)
}

pub fn flush(_evaler: &mut Evaler, file: &mut Value) -> Value {
    if let Value::Instance(inst) = file {
        if let Type::User(decl) = &inst.ty {
            if decl == "File" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let uni = _evaler.universe().borrow();
                    let b = uni.get_vmref_ref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let crate::universe::VmRefData::File(f) = &mut *ref_box {
                            use std::io::Write;
                            if let Err(e) = f.get_mut().flush() {
                                return Value::Error(format!("Flush error: {}", e).into());
                            }
                            return Value::Nil;
                        }
                    }
                }
            }
        }
    }
    Value::Nil
}

pub fn flush_method(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    flush(_evaler, instance)
}
