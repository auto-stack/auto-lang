use std::collections::HashMap;
use crate::value::Value;


pub fn builtins() -> HashMap<String, Value> {
    let mut builtins = HashMap::new();
    builtins.insert("print".to_string(), Value::ExtFn(print));
    builtins
}

pub fn print(args: &Vec<Value>) -> Value {
    for (i, arg) in args.iter().enumerate()  {
        print!("{}", arg);
        if i < args.len() - 1 {
            print!(", ");
        }
    }
    Value::Void
}
