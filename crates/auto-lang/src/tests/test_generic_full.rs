use crate::parse_preserve_error;
use crate::error::{AutoError, attach_source};

#[test]
fn test_full_generic_usage() {
    // Test 1: Just tag definition
    let code1 = r#"tag May<T> { nil Nil, val T }"#;
    match parse_preserve_error(code1) {
        Ok(_) => println!("✓ Tag definition parsed"),
        Err(e) => {
            let err_with_src = attach_source(e, "test.at".to_string(), code1.to_string());
            eprintln!("✗ Tag definition failed: {}\n", err_with_src);
        }
    }

    // Test 2: Tag + function
    let code2 = r#"
tag May<T> { nil Nil, val T }

fn main() {
    42
}
"#;
    match parse_preserve_error(code2) {
        Ok(_) => println!("✓ Tag + function parsed"),
        Err(e) => {
            let err_with_src = attach_source(e, "test.at".to_string(), code2.to_string());
            eprintln!("✗ Tag + function failed: {}\n", err_with_src);
        }
    }

    // Test 3: let statement with type annotation
    let code3 = r#"
tag May<T> { nil Nil, val T }

fn main() {
    let x May<int>
}
"#;
    match parse_preserve_error(code3) {
        Ok(_) => println!("✓ let with generic type parsed"),
        Err(e) => {
            let err_with_src = attach_source(e, "test.at".to_string(), code3.to_string());
            eprintln!("✗ let with generic type failed:\n");

            match &err_with_src {
                AutoError::MultipleErrors { count, errors, .. } => {
                    eprintln!("Multiple errors ({}):\n", count);
                    for (i, err) in errors.iter().enumerate() {
                        eprintln!("--- Error {} ---", i + 1);
                        eprintln!("{}\n", err);
                    }
                }
                _ => {
                    eprintln!("{}\n", err_with_src);
                }
            }
        }
    }

    // Test 4: May.val(42) call
    let code4 = r#"
tag May<T> { nil Nil, val T }

fn main() {
    May.val(42)
}
"#;
    match parse_preserve_error(code4) {
        Ok(_) => println!("✓ May.val(42) parsed"),
        Err(e) => {
            let err_with_src = attach_source(e, "test.at".to_string(), code4.to_string());
            eprintln!("✗ May.val(42) failed: {}\n", err_with_src);
        }
    }
}
