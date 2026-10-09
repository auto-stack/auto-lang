use super::context::VmContext;
use auto_val::{Instance, Obj, Type, Value};
use std::{
    fs::File,
    io::{BufRead, Read},
};

/// Producer-owned logical contracts for VmModule dispatch. Receiver is carried
/// separately by VmMethod; parameters here are the actual `args` payload.
pub fn method_contract(method: &str) -> Option<crate::vm::native::NativeContract> {
    let (parameters, returns): (&[&str], &str) = match method {
        "read_text" | "read_line" => (&[], "str"),
        "read_char" => (&[], "int"),
        // read_buf_method is a stub; it cannot supply a supported contract.
        "read_buf" => return None,
        "write_line" => (&["str"], "void"),
        "flush" | "close" => (&[], "void"),
        "open" => (&["str"], "File"),
        _ => return None,
    };
    Some(crate::vm::native::NativeContract {
        parameters: parameters.iter().map(|p| p.to_string()).collect(),
        returns: returns.into(), producer: format!("vm::io::{method}"),
        receiver: (method != "open").then(|| "File".into()),
        is_static: method == "open", generics: Vec::new(),
        parameter_modes: parameters.iter().map(|_| "View".into()).collect(),
        error_shape: "Value::Error".into(),
    })
}

pub fn selected_method_contract(method: &str, selected: super::VmMethod) -> Option<crate::vm::native::NativeContract> {
    let producer: super::VmMethod = match method {
        "read_text" => read_text_method, "read_line" => read_line_method,
        "read_char" => read_char_method, "write_line" => write_line_method,
        "flush" => flush_method, "close" => close_method,
        _ => return None,
    };
    std::ptr::fn_addr_eq(producer, selected).then(|| method_contract(method)).flatten()
}


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
                            Value::Instance(Box::new(Instance {
                                ty: auto_val::Type::from(ty),
                                fields,
                            }))
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
                            Value::Instance(Box::new(Instance {
                                ty: auto_val::Type::from(ty),
                                fields,
                            }))
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
