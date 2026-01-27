// Test Plan 057: Generic Spec Parsing
use crate::run;

#[test]
fn test_parse_generic_spec() {
    let code = r#"
        spec Storage<T> {
            fn get() T
            fn set(value T)
        }

        fn main() {
            return 0
        }
    "#;

    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    // The parsing should succeed (no "syntax error")
    assert!(!result.contains("syntax error"),
        "Generic spec parsing should succeed, got: {}", result);
}

#[test]
fn test_parse_generic_spec_impl() {
    let code = r#"
        spec Storage<T> {
            fn get() T
        }

        type Heap<T> as Storage<T> {
            ptr *T
        }

        fn main() {
            return 0
        }
    "#;

    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    // The parsing should succeed (no "syntax error")
    assert!(!result.contains("syntax error"),
        "Generic spec impl parsing should succeed, got: {}", result);
}

#[test]
fn test_parse_non_generic_spec() {
    let code = r#"
        spec Flyer {
            fn fly()
        }

        type Pigeon as Flyer {
            name str
        }

        fn main() {
            return 0
        }
    "#;

    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    // The parsing should succeed (no "syntax error")
    assert!(!result.contains("syntax error"),
        "Non-generic spec parsing should succeed, got: {}", result);
}

#[test]
fn test_parse_generic_spec_with_multiple_params() {
    let code = r#"
        spec Map<K, V> {
            fn get(key K) V
            fn set(key K, value V)
        }

        fn main() {
            return 0
        }
    "#;

    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    // The parsing should succeed (no "syntax error")
    assert!(!result.contains("syntax error"),
        "Generic spec with multiple params parsing should succeed, got: {}", result);
}

#[test]
fn test_parse_generic_spec_with_const_param() {
    let code = r#"
        spec Buffer<T, N uint> {
            fn size() uint
            fn get(index uint) T
        }

        fn main() {
            return 0
        }
    "#;

    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    // The parsing should succeed (no "syntax error")
    assert!(!result.contains("syntax error"),
        "Generic spec with const param parsing should succeed, got: {}", result);
}

#[test]
fn test_parse_type_with_multiple_generic_specs() {
    let code = r#"
        spec Reader<T> {
            fn read() T
        }

        spec Writer<T> {
            fn write(value T)
        }

        type IO<T> as Reader<T>, Writer<T> {
            data *T
        }

        fn main() {
            return 0
        }
    "#;

    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    // The parsing should succeed (no "syntax error")
    assert!(!result.contains("syntax error"),
        "Type with multiple generic specs parsing should succeed, got: {}", result);
}
