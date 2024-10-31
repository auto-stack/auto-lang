use std::collections::HashMap;
use autoval::value::{Value, ExtFn};


pub fn builtins() -> HashMap<String, Value> {
    let mut builtins = HashMap::new();
    builtins.insert("print".to_string(), Value::ExtFn(ExtFn { fun: print }));
    builtins
}

pub fn print(args: &Vec<Value>) -> Value {
    for (i, arg) in args.iter().enumerate()  {
        print!("{}", arg);
        if i < args.len() - 1 {
            print!(", ");
        }
    }
    println!();
    Value::Void
}
