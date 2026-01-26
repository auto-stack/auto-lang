use auto_val::{Args, AutoStr, ExtFn, Value};
use std::collections::HashMap;
use std::cell::RefCell;

// Test output capture support
thread_local! {
    static TEST_OUTPUT_CAPTURE: RefCell<Option<std::sync::Arc<std::sync::Mutex<Vec<u8>>>>> = RefCell::new(None);
}

/// Enable test mode and return a buffer for capturing output
#[cfg(test)]
pub fn enable_test_capture() -> std::sync::Arc<std::sync::Mutex<Vec<u8>>> {
    use std::sync::{Arc, Mutex};

    let buffer = Arc::new(Mutex::new(Vec::new()));
    TEST_OUTPUT_CAPTURE.with(|capture| {
        *capture.borrow_mut() = Some(buffer.clone());
    });
    buffer
}

/// Disable test mode and clear the capture buffer
#[cfg(test)]
pub fn disable_test_capture() {
    TEST_OUTPUT_CAPTURE.with(|capture| {
        *capture.borrow_mut() = None;
    });
}

/// Get the captured output as a string
#[cfg(test)]
pub fn get_captured_output(buffer: &std::sync::Arc<std::sync::Mutex<Vec<u8>>>) -> String {
    let data = buffer.lock().unwrap();
    String::from_utf8(data.clone()).unwrap_or_default()
}

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

    let name: AutoStr = "str_lines".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_lines, name }));

    let name: AutoStr = "str_words".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::string::str_words, name }));

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

    // Option functions (Plan 027)
    let name: AutoStr = "Option_some".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::result::option_some, name }));

    let name: AutoStr = "Option_none".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::result::option_none, name }));

    let name: AutoStr = "Option_is_some".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::result::option_is_some, name }));

    let name: AutoStr = "Option_is_none".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::result::option_is_none, name }));

    let name: AutoStr = "Option_unwrap".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::result::option_unwrap, name }));

    let name: AutoStr = "Option_unwrap_or".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::result::option_unwrap_or, name }));

    let name: AutoStr = "Option_unwrap_or_null".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::result::option_unwrap_or_null, name }));

    // Result functions (Plan 027)
    let name: AutoStr = "Result_ok".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::result::result_ok, name }));

    let name: AutoStr = "Result_err".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::result::result_err, name }));

    let name: AutoStr = "Result_is_ok".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::result::result_is_ok, name }));

    let name: AutoStr = "Result_is_err".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::result::result_is_err, name }));

    let name: AutoStr = "Result_unwrap".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::result::result_unwrap, name }));

    let name: AutoStr = "Result_unwrap_err".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::result::result_unwrap_err, name }));

    let name: AutoStr = "Result_unwrap_or".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::result::result_unwrap_or, name }));

    let name: AutoStr = "Result_unwrap_err_or".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::result::result_unwrap_err_or, name }));

    // May functions (Plan 027 Phase 1b)
    let name: AutoStr = "May_empty".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::may::may_empty, name }));

    let name: AutoStr = "May_value".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::may::may_value, name }));

    let name: AutoStr = "May_error".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::may::may_error, name }));

    let name: AutoStr = "May_is_empty".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::may::may_is_empty, name }));

    let name: AutoStr = "May_is_value".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::may::may_is_value, name }));

    let name: AutoStr = "May_is_error".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::may::may_is_error, name }));

    let name: AutoStr = "May_unwrap".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::may::may_unwrap, name }));

    let name: AutoStr = "May_unwrap_or".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::may::may_unwrap_or, name }));

    let name: AutoStr = "May_unwrap_or_null".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::may::may_unwrap_or_null, name }));

    let name: AutoStr = "May_unwrap_error".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::may::may_unwrap_error, name }));

    let name: AutoStr = "May_unwrap_error_or".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::may::may_unwrap_error_or, name }));

    // File I/O methods (Plan 036 Phase 4)
    let name: AutoStr = "file_read_all".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::file::file_read_all, name }));

    let name: AutoStr = "file_write_lines".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::file::file_write_lines, name }));

    // System functions
    let name: AutoStr = "getpid".into();
    builtins.insert(name.clone(), Value::ExtFn(ExtFn { fun: crate::libs::sys::sys_getpid, name }));

    builtins
}

// TODO: fix for named args
pub fn print(args: &Args) -> Value {
    eprintln!("DEBUG print: Called with args = {:?}", args);
    // Check if we're in test mode
    let test_capture = TEST_OUTPUT_CAPTURE.with(|capture| capture.borrow().clone());

    if let Some(buffer) = test_capture {
        // Test mode: write to buffer
        let mut output = String::new();
        for (i, arg) in args.args.iter().enumerate() {
            // Get the value from the Arg
            let value = arg.get_val();
            // Use repr() for strings to avoid quotes
            output.push_str(&value.repr());
            if i < args.args.len() - 1 {
                output.push(' ');
            }
        }
        output.push('\n');

        // Write to buffer
        let mut buf = buffer.lock().unwrap();
        buf.extend_from_slice(output.as_bytes());
    } else {
        // Normal mode: write to stdout
        use std::io::{self, Write};

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        for (i, arg) in args.args.iter().enumerate() {
            // Get the value from the Arg
            let value = arg.get_val();
            // Use repr() for strings to avoid quotes
            write!(handle, "{}", value.repr()).ok();
            if i < args.args.len() - 1 {
                write!(handle, " ").ok();
            }
        }
        writeln!(handle).ok();
        handle.flush().ok();
    }

    Value::Void
}
