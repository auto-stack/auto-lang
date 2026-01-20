use crate::error::AutoResult;
use crate::run;

// ===== Standard Library Tests (Interpreter Counterparts to a2c Tests) =====
// NOTE: Some tests are marked as ignore because they require stdlib import support
// which is not yet fully implemented in the interpreter. These tests will be enabled
// once the stdlib loading mechanism is complete.
/// Helper function to run code and capture stdout output
fn run_with_output(code: &str) -> AutoResult<(String, String)> {
    use crate::libs::builtin::{disable_test_capture, enable_test_capture, get_captured_output};

    // Enable test capture
    let buffer = enable_test_capture();

    // Run the code
    let result = run(code);

    // Get captured output before disabling capture
    let output = get_captured_output(&buffer);

    // Disable test capture
    disable_test_capture();

    // Return result and output
    Ok((result?, output))
}

#[test]
fn test_std_io_say() {
    // Test auto.io: say function (interpreter version of a2c/100_std_hello)
    let code = r#"
    use auto.io: say

    say("hello!")
    "#;
    let (result, output) = run_with_output(code).unwrap();
    assert_eq!(result, "");
    assert_eq!(output, "hello!\n");
}

#[test]
fn test_std_io_say_multiple() {
    // Test multiple say calls
    let code = r#"
    use auto.io: say

    say("line 1")
    say("line 2")
    say("line 3")
    "#;
    let (result, output) = run_with_output(code).unwrap();
    assert_eq!(result, "");
    assert_eq!(output, "line 1\nline 2\nline 3\n");
}

#[test]
fn test_std_sys_get_pid() {
    // Test auto.sys: get_pid function (interpreter version of a2c/101_std_getpid)
    let code = r#"
    use auto.sys: getpid

    let pid = getpid()
    pid
    "#;
    let result = run(code).unwrap();
    // PID should be a positive integer
    assert!(result.parse::<i64>().is_ok());
    let pid_val = result.parse::<i64>().unwrap();
    println!("PID: {}", pid_val);
    assert!(pid_val > 0);
}

#[test]
#[ignore = "requires stdlib import support (use auto.str)"]
fn test_std_str_str() {
    // Test auto.str: str function (interpreter version of a2c/105_std_str)
    let code = r#"
    use auto.str: str

    let s1 str = str("Hello")
    s1
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Hello");
}

#[test]
#[ignore = "requires stdlib import support (use auto.str)"]
fn test_std_str_str_with_literal() {
    // Test str function with string literal
    let code = r#"
    use auto.str: str

    let s str = str("World")
    s
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "World");
}

#[test]
fn test_std_io_print() {
    // Test print function with multiple arguments
    let code = r#"
    print("Hello", "World", 42)
    "#;
    let (result, output) = run_with_output(code).unwrap();
    assert_eq!(result, "");
    assert_eq!(output, "Hello World 42\n");
}

#[test]
fn test_std_io_print_with_vars() {
    // Test print with variables
    let code = r#"
    let name = "Alice"
    let age = 30
    print(name, age)
    "#;
    let (result, output) = run_with_output(code).unwrap();
    assert_eq!(result, "");
    assert_eq!(output, "Alice 30\n");
}

#[test]
fn test_std_io_print_number() {
    // Test print with numbers
    let code = r#"
    print(123)
    print(456)
    "#;
    let (result, output) = run_with_output(code).unwrap();
    assert_eq!(result, "");
    assert_eq!(output, "123\n456\n");
}

#[test]
fn test_std_io_print_bool() {
    // Test print with boolean values
    let code = r#"
    print(true)
    print(false)
    "#;
    let (result, output) = run_with_output(code).unwrap();
    assert_eq!(result, "");
    assert_eq!(output, "true\nfalse\n");
}

#[test]
fn test_std_io_print_array() {
    // Test print with arrays
    let code = r#"
    let arr = [1, 2, 3]
    print(arr)
    "#;
    let (result, output) = run_with_output(code).unwrap();
    assert_eq!(result, "");
    assert_eq!(output, "[1, 2, 3]\n");
}

#[test]
fn test_std_io_print_object() {
    // Test print with objects
    let code = r#"
    let obj = {name: "Bob", age: 25}
    print(obj)
    "#;
    let (result, output) = run_with_output(code).unwrap();
    assert_eq!(result, "");
    assert_eq!(output, "{name: \"Bob\", age: 25}\n");
}

#[test]
fn test_std_use_combined() {
    // Test combining multiple std imports
    let code = r#"
    use auto.io: say, print
    use auto.sys: getpid

    say("Testing std")
    let pid = getpid()
    print("PID:", pid)
    "#;
    let (result, output) = run_with_output(code).unwrap();
    assert_eq!(result, "");
    // Output should contain "Testing std" and "PID: <number>"
    println!("Output: {}", output);
    assert!(output.contains("Testing std\n"));
    assert!(output.contains("PID:"));
}

#[test]
fn test_std_math_functions() {
    // Test auto.math functions (already tested in test_std, but verify again)
    let code = r#"
    use auto.math: square, cube

    square(5) + cube(3)
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "52"); // 25 + 27 = 52
}

#[test]
#[ignore = "requires stdlib import support (upper/lower functions not in auto.str)"]
fn test_std_str_functions() {
    // Test various string functions from auto.str
    let code = r#"
    use auto.str: upper, lower

    let s = "Hello"
    upper(s) == "HELLO" && lower(s) == "hello"
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "1");
}

#[test]
fn test_std_test() {
    let code = r#"use auto.test: test
        test()"#;
    let result = run(code).unwrap();
    assert_eq!(result, "42");
}

#[test]
fn test_std_file() {
    let code = r#"use auto.io: File
    let f File = File.open("Cargo.toml")
    let s = f.read_text()
    f.close()
    s
        "#;
    let result = run(code).unwrap();
    println!("Result: {}", result);
}

#[test]
fn test_std_file_readline() {
    let code = r#"use auto.io: File
    let f File = File.open("../../test/txt/test_lines.txt")
    let line1 = f.read_line()
    let line2 = f.read_line()
    f.close()
    line1
        "#;
    let result = run(code).unwrap();
    println!("Result: {}", result);
    assert_eq!(result, "First line");
}

#[test]
fn test_std_file_readchar() {
    let code = r#"use auto.io: File
    let f File = File.open("../../test/txt/test_lines.txt")
    let ch = f.read_char()
    f.close()
    ch
        "#;
    let result = run(code).unwrap();
    println!("Result: {}", result);
    assert_eq!(result, "70");
}

#[test]
fn test_std_file_flush() {
    // We generally can't verify side-effects easily on read-only files,
    // but we can verify that the method is callable and doesn't panic.
    // File.open currently opens in read-only mode.
    // Flush on a read-only file buffer (BufReader) might be a no-op or valid.
    let code = r#"use auto.io: File
        let f = File.open("Cargo.toml")
        f.flush()
        f.close()
        "OK"
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "OK");
}


#[test]
fn test_std_file_write_line() {
    // This test attempts to look up the function to ensure it's registered
    // Actual execution might fail due to read-only open in VM, but we check if code runs/compiles
    let _code = r#"
            use auto.io: File
            fn main() {
                // Just check if we can call it without method reference error
                // We pass a dummy file object if possible or just parse/analyze
                // But run() executes.
                // Let's rely on the fact that if method is missing, it returns error.
                // If it exists, it might error on "write error" or "file not open for write"
                // which confirms the method was called.
            }
        "#;
    // Since we can't easily mock a writable file in the current VM setup without more changes:
    // We'll verify the method exists in the registry directly.

    crate::vm::init_io_module();
    let registry = crate::vm::VM_REGISTRY.lock().unwrap();
    let method = registry.get_method("File", "write_line");
    assert!(
        method.is_some(),
        "File.write_line method should be registered"
    );
}
