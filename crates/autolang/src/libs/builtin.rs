use std::collections::HashMap;
use autoval::{Value, ExtFn, Args};


pub fn builtins() -> HashMap<String, Value> {
    let mut builtins = HashMap::new();
    builtins.insert("print".to_string(), Value::ExtFn(ExtFn { fun: print }));
    builtins
}

// TODO: fix for named args
pub fn print(args: &Args) -> Value {
    for (i, arg) in args.array.iter().enumerate()  {
        print!("{}", arg);
        if i < args.array.len() - 1 {
            print!(", ");
        }
    }
    if args.named.len() > 0 {
        for (key, value) in args.named.iter() {
            print!(", {}:{}", key, value);
        }
    }
    println!();
    Value::Void
}

