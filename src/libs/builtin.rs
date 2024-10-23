use std::collections::HashMap;
use crate::value::Value;


pub fn builtins() -> HashMap<String, Value> {
    let mut builtins = HashMap::new();
    builtins.insert("print".to_string(), Value::ExtFn(print));
    builtins
}

pub fn print(args: &Vec<Value>) -> Value {
    for arg in args {
        println!("{}", arg);
    }
    Value::Void
}
