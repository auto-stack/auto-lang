use std::collections::HashMap;
use autoval::{Value, ExtFn, Args};


pub fn builtins() -> HashMap<String, Value> {
    let mut builtins = HashMap::new();
    let name = "print".to_string();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: print, name }));
    builtins
}

// TODO: fix for named args
pub fn print(args: &Args) -> Value {
    for (i, arg) in args.args.iter().enumerate()  {
        print!("{}", arg);
        if i < args.args.len() - 1 {
            print!(", ");
        }
    }
    println!();
    Value::Void
}

