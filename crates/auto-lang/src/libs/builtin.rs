use auto_val::{Args, AutoStr, ExtFn, Value};
use std::collections::HashMap;

pub fn builtins() -> HashMap<AutoStr, Value> {
    let mut builtins = HashMap::new();

    // Print function
    let name: AutoStr = "print".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: print, name }));

    // String functions - Basic
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

    // String functions - Search (Plan 025)
    let name: AutoStr = "str_contains".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_contains, name }));

    let name: AutoStr = "str_starts_with".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_starts_with, name }));

    let name: AutoStr = "str_ends_with".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_ends_with, name }));

    let name: AutoStr = "str_find".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_find, name }));

    // String functions - Transform (Plan 025)
    let name: AutoStr = "str_trim".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_trim, name }));

    let name: AutoStr = "str_trim_left".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_trim_left, name }));

    let name: AutoStr = "str_trim_right".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_trim_right, name }));

    let name: AutoStr = "str_replace".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_replace, name }));

    // String functions - Split/Join (Plan 025)
    let name: AutoStr = "str_split".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_split, name }));

    let name: AutoStr = "str_join".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_join, name }));

    // String functions - Compare (Plan 025)
    let name: AutoStr = "str_compare".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_compare, name }));

    let name: AutoStr = "str_eq_ignore_case".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_eq_ignore_case, name }));

    // String functions - Utilities (Plan 025)
    let name: AutoStr = "str_repeat".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_repeat, name }));

    let name: AutoStr = "str_char_at".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_char_at, name }));

    // String slice functions (Phase 3)
    let name: AutoStr = "as_slice".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_slice, name }));

    let name: AutoStr = "slice_len".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_slice_len, name }));

    let name: AutoStr = "slice_get".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_slice_get, name }));

    // C FFI functions (Plan 025)
    let name: AutoStr = "cstr_new".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::cstr_new, name }));

    let name: AutoStr = "cstr_len".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::cstr_len, name }));

    let name: AutoStr = "cstr_as_ptr".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::cstr_as_ptr, name }));

    let name: AutoStr = "cstr_to_str".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::cstr_to_str, name }));

    let name: AutoStr = "to_cstr".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::to_cstr, name }));

    builtins
}

// TODO: fix for named args
pub fn print(args: &Args) -> Value {
    use std::io::{self, Write};

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    for (i, arg) in args.args.iter().enumerate() {
        write!(handle, "{}", arg).ok();
        if i < args.args.len() - 1 {
            write!(handle, ", ").ok();
        }
    }
    writeln!(handle).ok();
    handle.flush().ok();

    Value::Void
}
