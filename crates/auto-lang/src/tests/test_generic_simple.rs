use crate::parse_preserve_error;
use crate::error::{AutoError, attach_source};

#[test]
fn test_simple_generic_tag() {
    // Test 1: Just the tag definition
    let code1 = r#"tag May<T> { nil Nil }"#;
    match parse_preserve_error(code1) {
        Ok(_) => println!("✓ Simple generic tag parsed successfully"),
        Err(e) => {
            let err_with_src = attach_source(e, "test.at".to_string(), code1.to_string());
            eprintln!("✗ Simple generic tag failed:\n{}\n", err_with_src);
        }
    }

    // Test 2: Tag with type parameter used in field
    let code2 = r#"tag May<T> { nil Nil, val T }"#;
    match parse_preserve_error(code2) {
        Ok(_) => println!("✓ Generic tag with type param field parsed successfully"),
        Err(e) => {
            let err_with_src = attach_source(e, "test.at".to_string(), code2.to_string());
            eprintln!("✗ Generic tag with type param field failed:\n{}\n", err_with_src);
        }
    }

    // Test 3: Full example
    let code3 = r#"
tag May<T> {
    nil Nil
    val T
}
"#;
    match parse_preserve_error(code3) {
        Ok(_) => println!("✓ Full generic tag parsed successfully"),
        Err(e) => {
            let err_with_src = attach_source(e, "test.at".to_string(), code3.to_string());
            eprintln!("✗ Full generic tag failed:\n{}\n", err_with_src);
            match &err_with_src {
                AutoError::MultipleErrors { errors, .. } => {
                    for (i, err) in errors.iter().enumerate() {
                        eprintln!("  Error {}: {}\n", i + 1, err);
                    }
                }
                _ => {}
            }
        }
    }
}
