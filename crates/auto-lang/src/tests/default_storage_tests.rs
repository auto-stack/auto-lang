use crate::parse_preserve_error;
use crate::error::attach_source;

/// Test that recommended PC pattern (List with Heap storage) can be parsed
#[test]
fn test_pc_pattern_parses() {
    let code = r#"
        // Recommended pattern for PC: List with Heap storage
        type List<T, S> {
            len u32
            store S
        }

        type Heap<T> {
            ptr *T
            cap u32
        }

        fn main() {
            return 0
        }
    "#;

    match parse_preserve_error(code) {
        Ok(_) => println!("✓ PC pattern (List<T, Heap>) parses successfully"),
        Err(e) => {
            let err_with_src = attach_source(e, "test.at".to_string(), code.to_string());
            eprintln!("✗ PC pattern failed to parse:\n{}\n", err_with_src);
            panic!("Failed to parse PC pattern");
        }
    }
}

/// Test that recommended MCU pattern (List with InlineInt64) can be parsed
#[test]
fn test_mcu_pattern_parses() {
    let code = r#"
        // Recommended pattern for MCU: List with InlineInt64 storage
        type List<T, S> {
            len u32
            store S
        }

        type InlineInt64 {
            buffer [64]int
        }

        fn main() {
            return 0
        }
    "#;

    match parse_preserve_error(code) {
        Ok(_) => println!("✓ MCU pattern (List<T, InlineInt64>) parses successfully"),
        Err(e) => {
            let err_with_src = attach_source(e, "test.at".to_string(), code.to_string());
            eprintln!("✗ MCU pattern failed to parse:\n{}\n", err_with_src);
            panic!("Failed to parse MCU pattern");
        }
    }
}

/// Test that Storage spec and InlineInt64 can be parsed together
#[test]
fn test_inline_storage_with_spec_parses() {
    let code = r#"
        // Complete example with Storage spec and InlineInt64 implementation
        spec Storage<T> {
            fn data() *T
            fn capacity() u32
            fn try_grow(min_cap u32) bool
        }

        type InlineInt64 as Storage<int> {
            buffer [64]int
        }

        fn main() {
            return 0
        }
    "#;

    match parse_preserve_error(code) {
        Ok(_) => println!("✓ Inline storage with spec parses successfully"),
        Err(e) => {
            let err_with_src = attach_source(e, "test.at".to_string(), code.to_string());
            eprintln!("✗ Inline storage failed to parse:\n{}\n", err_with_src);
            panic!("Failed to parse inline storage");
        }
    }
}

/// Test that generic List with two parameters parses correctly
#[test]
fn test_generic_list_two_params_parses() {
    let code = r#"
        // List with element type and storage type
        type List<T, S> {
            len u32
            store S
        }

        fn main() {
            return 0
        }
    "#;

    match parse_preserve_error(code) {
        Ok(_) => println!("✓ Generic List<T, S> parses successfully"),
        Err(e) => {
            let err_with_src = attach_source(e, "test.at".to_string(), code.to_string());
            eprintln!("✗ Generic List failed to parse:\n{}\n", err_with_src);
            panic!("Failed to parse generic List");
        }
    }
}

/// Test that explicit generic type annotations can be parsed
#[test]
fn test_explicit_generic_type_parses() {
    let code = r#"
        type List<T, S> {
            len u32
            store S
        }

        fn main() {
            let list List<int, Heap>
            return 0
        }
    "#;

    match parse_preserve_error(code) {
        Ok(_) => println!("✓ Explicit generic type annotation parses successfully"),
        Err(e) => {
            let err_with_src = attach_source(e, "test.at".to_string(), code.to_string());
            eprintln!("✗ Explicit generic type failed to parse:\n{}\n", err_with_src);
            panic!("Failed to parse explicit generic type");
        }
    }
}
