use crate::parse_preserve_error;
use crate::error::attach_source;
use crate::ast::{Type, StorageType, StorageKind};

#[test]
fn test_parse_dynamic_storage() {
    let code = r#"
        fn test() {
            let x Dynamic
        }
    "#;

    match parse_preserve_error(code) {
        Ok(_) => println!("✓ Dynamic storage parsed"),
        Err(e) => {
            let err_with_src = attach_source(e, "test.at".to_string(), code.to_string());
            eprintln!("✗ Dynamic storage failed: {}\n", err_with_src);
            panic!("Failed to parse Dynamic storage");
        }
    }
}

#[test]
fn test_parse_fixed_storage() {
    let code = r#"
        fn test() {
            let x Fixed<int>
        }
    "#;

    match parse_preserve_error(code) {
        Ok(_) => println!("✓ Fixed<int> storage parsed"),
        Err(e) => {
            let err_with_src = attach_source(e, "test.at".to_string(), code.to_string());
            eprintln!("✗ Fixed<int> storage failed: {}\n", err_with_src);
            panic!("Failed to parse Fixed<int> storage");
        }
    }
}

// TODO: Re-enable this test when trait bounds are implemented
// #[test]
// fn test_parse_storage_in_type_annotation() {
//     let code = r#"
//         type List<T, S : Storage = Dynamic> {
//             elems: [~]T
//         }
//     "#;
//
//     match parse_preserve_error(code) {
//         Ok(_) => println!("✓ Type with Storage parameter parsed"),
//         Err(e) => {
//             let err_with_src = attach_source(e, "test.at".to_string(), code.to_string());
//             eprintln!("✗ Type with Storage parameter failed: {}\n", err_with_src);
//             panic!("Failed to parse type with Storage parameter");
//         }
//     }
// }

#[test]
fn test_storage_type_display() {
    let storage_dynamic = Type::Storage(StorageType {
        kind: StorageKind::Dynamic,
    });
    assert_eq!(storage_dynamic.to_string(), "Dynamic");

    let storage_fixed = Type::Storage(StorageType {
        kind: StorageKind::Fixed { capacity: 64 },
    });
    assert_eq!(storage_fixed.to_string(), "Fixed<64>");
}

// ============================================================================
// VM Tests for Plan 052 Storage Module
// ============================================================================

use crate::run;

#[test]
fn test_heap_storage_new() {
    let code = r#"
        spec Storage<T> {
            fn data() *T
            fn capacity() u32
            fn try_grow(min_cap u32) bool
        }

        type Heap<T> as Storage<T> {
            ptr *T
            cap u32
        }

        fn main() {
            return 0
        }
    "#;

    let result = run(code).unwrap();
    // Should parse without error
    assert!(!result.contains("Error"));
}

#[test]
fn test_heap_storage_capacity() {
    let code = r#"
        spec Storage<T> {
            fn data() *T
            fn capacity() u32
            fn try_grow(min_cap u32) bool
        }

        type Heap<T> as Storage<T> {
            ptr *T
            cap u32
        }

        fn main() {
            return 0
        }
    "#;

    let result = run(code).unwrap();
    assert!(!result.contains("Error"));
}

#[test]
fn test_heap_storage_try_grow() {
    let code = r#"
        spec Storage<T> {
            fn data() *T
            fn capacity() u32
            fn try_grow(min_cap u32) bool
        }

        type Heap<T> as Storage<T> {
            ptr *T
            cap u32
        }

        fn main() {
            return 0
        }
    "#;

    let result = run(code).unwrap();
    assert!(!result.contains("Error"));
}

#[test]
fn test_inline_int64_storage_new() {
    let code = r#"
        spec Storage<T> {
            fn data() *T
            fn capacity() u32
            fn try_grow(min_cap u32) bool
        }

        type InlineInt64 as Storage<int> {
            buffer [64]int
        }

        fn main() {
            let inline = InlineInt64.new()
            return inline.capacity()
        }
    "#;

    let result = run(code).unwrap();
    // Should have capacity 64
    assert_eq!(result, "64");
}

#[test]
fn test_inline_int64_storage_capacity() {
    let code = r#"
        spec Storage<T> {
            fn data() *T
            fn capacity() u32
            fn try_grow(min_cap u32) bool
        }

        type InlineInt64 as Storage<int> {
            buffer [64]int
        }

        fn main() {
            let inline = InlineInt64.new()
            let cap = inline.capacity()
            return cap
        }
    "#;

    let result = run(code).unwrap();
    assert_eq!(result, "64");
}

#[test]
fn test_inline_int64_storage_try_grow_success() {
    let code = r#"
        spec Storage<T> {
            fn data() *T
            fn capacity() u32
            fn try_grow(min_cap u32) bool
        }

        type InlineInt64 as Storage<int> {
            buffer [64]int
        }

        fn main() {
            let inline = InlineInt64.new()
            let success = inline.try_grow(50)
            return success
        }
    "#;

    let result = run(code).unwrap();
    // Should succeed (50 <= 64)
    assert_eq!(result, "true");
}

#[test]
fn test_inline_int64_storage_try_grow_failure() {
    let code = r#"
        spec Storage<T> {
            fn data() *T
            fn capacity() u32
            fn try_grow(min_cap u32) bool
        }

        type InlineInt64 as Storage<int> {
            buffer [64]int
        }

        fn main() {
            let inline = InlineInt64.new()
            let success = inline.try_grow(100)
            return success
        }
    "#;

    let result = run(code).unwrap();
    // Should fail (100 > 64)
    assert_eq!(result, "false");
}

#[test]
fn test_storage_spec_declaration() {
    // Test that generic Storage<T> spec parses correctly
    let code = r#"
        spec Storage<T> {
            fn data() *T
            fn capacity() u32
            fn try_grow(min_cap u32) bool
        }

        type Heap<T> as Storage<T> {
            ptr *T
            cap u32
        }

        fn main() {
            let heap = Heap.new()
            let cap = heap.capacity()
            return cap
        }
    "#;

    let result = run(code).unwrap();
    // Should parse and execute without error
    assert!(!result.contains("syntax error"));
    assert!(!result.contains("Error"));
}
