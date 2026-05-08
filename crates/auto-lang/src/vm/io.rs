use super::context::VmContext;
use auto_val::{Instance, Obj, Type, Value};
use std::{
    fs::File,
    io::{BufRead, Read},
};


pub fn open(ctx: &mut VmContext, path: Value) -> Value {
    match path {
        Value::Str(p) => {
            let f = File::open(p.as_str());
            match f {
                Ok(file) => {
                    let ty = ctx.lookup_type("File");
                    match &ty {
                        Type::User(_) => {
                            let reader = std::io::BufReader::new(file);
                            let id = ctx.add_vmref(super::types::VmRefData::File(reader));
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
        Value::String(p) => {
            let f = File::open(p.as_str());
            match f {
                Ok(file) => {
                    let ty = ctx.lookup_type("File");
                    match &ty {
                        Type::User(_) => {
                            let reader = std::io::BufReader::new(file);
                            let id = ctx.add_vmref(super::types::VmRefData::File(reader));
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

pub fn read_text(ctx: &mut VmContext, file: &mut Value) -> Value {
    if let Value::Instance(inst) = file {
        if let Type::User(decl) = &inst.ty {
            if decl == "File" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let b = ctx.get_vmref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let super::types::VmRefData::File(f) = &mut *ref_box {
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

pub fn read_line(ctx: &mut VmContext, file: &mut Value) -> Value {
    if let Value::Instance(inst) = file {
        if let Type::User(decl) = &inst.ty {
            if decl == "File" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let b = ctx.get_vmref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let super::types::VmRefData::File(f) = &mut *ref_box {
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

pub fn close(ctx: &mut VmContext, file: &mut Value) -> Value {
    if let Value::Instance(inst) = file {
        if let Type::User(decl) = &inst.ty {
            if decl == "File" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    ctx.drop_vmref(id);
                };
            }
        }
    }
    Value::Nil
}

/// Wrapper for read_text to match VmMethod signature
pub fn read_text_method(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    read_text(ctx, instance)
}

/// Wrapper for close to match VmMethod signature
pub fn close_method(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    close(ctx, instance)
}

pub fn read_line_method(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    read_line(ctx, instance)
}

pub fn read_char(ctx: &mut VmContext, file: &mut Value) -> Value {
    if let Value::Instance(inst) = file {
        if let Type::User(decl) = &inst.ty {
            if decl == "File" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let b = ctx.get_vmref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let super::types::VmRefData::File(f) = &mut *ref_box {
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

pub fn read_char_method(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    read_char(ctx, instance)
}

pub fn read_buf(_ctx: &mut VmContext, _file: &mut Value, _buf: &mut Value, _size: i64) -> Value {
    // VM does not support read_buf with mutable string buffer yet for immutable str
    Value::Int(0)
}

pub fn read_buf_method(_ctx: &mut VmContext, _instance: &mut Value, _args: Vec<Value>) -> Value {
    // Stub implementation
    Value::Int(0)
}

pub fn write_line(ctx: &mut VmContext, file: &mut Value, line: &str) -> Value {
    if let Value::Instance(inst) = file {
        if let Type::User(decl) = &inst.ty {
            if decl == "File" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let b = ctx.get_vmref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let super::types::VmRefData::File(f) = &mut *ref_box {
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

pub fn write_line_method(ctx: &mut VmContext, instance: &mut Value, args: Vec<Value>) -> Value {
    let line = if let Some(val) = args.get(0) {
        match val {
            Value::Str(s) => s.as_str(),
            Value::String(s) => s.as_str(),
            _ => return Value::Error("Argument must be a string".into()),
        }
    } else {
        return Value::Error("Missing argument".into());
    };
    write_line(ctx, instance, line)
}

pub fn flush(ctx: &mut VmContext, file: &mut Value) -> Value {
    if let Value::Instance(inst) = file {
        if let Type::User(decl) = &inst.ty {
            if decl == "File" {
                let id = inst.fields.get("id");
                if let Some(Value::USize(id)) = id {
                    let b = ctx.get_vmref(id);
                    if let Some(b) = b {
                        let mut ref_box = b.borrow_mut();
                        if let super::types::VmRefData::File(f) = &mut *ref_box {
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

pub fn flush_method(ctx: &mut VmContext, instance: &mut Value, _args: Vec<Value>) -> Value {
    flush(ctx, instance)
}
