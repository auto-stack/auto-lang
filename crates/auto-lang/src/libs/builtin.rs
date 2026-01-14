use auto_val::{Args, AutoStr, ExtFn, Value};
use std::collections::HashMap;

pub fn builtins() -> HashMap<AutoStr, Value> {
    let mut builtins = HashMap::new();

    // Print function
    let name: AutoStr = "print".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: print, name }));

    // String functions
    let name: AutoStr = "str_new".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_new, name }));

    let name: AutoStr = "str_len".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_len, name }));

    let name: AutoStr = "str_append".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_append, name }));

    let name: AutoStr = "str_upper".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_upper, name }));

    let name: AutoStr = "str_lower".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_lower, name }));

    let name: AutoStr = "str_sub".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_sub, name }));

    builtins
}

// TODO: fix for named args
pub fn print(args: &Args) -> Value {
    for (i, arg) in args.args.iter().enumerate() {
        print!("{}", arg);
        if i < args.args.len() - 1 {
            print!(", ");
        }
    }
    println!();
    Value::Void
}
